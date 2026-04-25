use sqlx::SqlitePool;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

struct LoadedEntry {
    bytes: Vec<u8>,
    mime: &'static str,
    base_href: String,
    resolved_href: String,
    is_html: bool,
}

pub fn epub_handler(
    app: &AppHandle,
    req: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    match handle(app, req) {
        Ok(resp) => resp,
        Err(_) => tauri::http::Response::builder()
            .status(404)
            .body(b"not found".to_vec())
            .unwrap_or_else(|_| tauri::http::Response::new(b"not found".to_vec())),
    }
}

fn handle(
    app: &AppHandle,
    req: tauri::http::Request<Vec<u8>>,
) -> Result<tauri::http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let uri = req.uri();
    let (job_id, href) = split_job_id_and_href(uri)?;

    let pool = app.state::<Arc<SqlitePool>>();
    let file_path: Option<String> = tauri::async_runtime::block_on(async {
        sqlx::query_as::<_, (String,)>("SELECT file_path FROM jobs WHERE id = ?")
            .bind(&job_id)
            .fetch_optional(pool.inner().as_ref())
            .await
            .ok()
            .flatten()
            .map(|(p,)| p)
    });

    let file_path = file_path.ok_or("job not found")?;
    let loaded = load_entry(&file_path, &job_id, &href)?;
    let body = if loaded.is_html {
        decorate_html(
            &String::from_utf8_lossy(&loaded.bytes),
            &job_id,
            &loaded.base_href,
            &loaded.resolved_href,
        )
        .into_bytes()
    } else {
        loaded.bytes
    };
    let resp = tauri::http::Response::builder()
        .status(200)
        .header("Content-Type", loaded.mime)
        .body(body)?;
    Ok(resp)
}

pub fn render_html(
    file_path: &str,
    job_id: &str,
    href: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let loaded = load_entry(file_path, job_id, href)?;
    if !loaded.is_html {
        return Err("requested asset is not HTML".into());
    }

    Ok(decorate_html(
        &String::from_utf8_lossy(&loaded.bytes),
        job_id,
        &loaded.base_href,
        &loaded.resolved_href,
    ))
}

fn load_entry(
    file_path: &str,
    job_id: &str,
    href: &str,
) -> Result<LoadedEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))?;

    let opf_path = {
        let mut container = archive.by_name("META-INF/container.xml")?;
        let mut xml = String::new();
        container.read_to_string(&mut xml)?;
        let doc = roxmltree::Document::parse(&xml)?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .ok_or("no rootfile")?
            .to_string()
    };
    let opf_dir = Path::new(&opf_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let resolved_href = if archive.by_name(href).is_ok() || opf_dir.as_os_str().is_empty() {
        href.to_string()
    } else {
        opf_dir.join(href).to_string_lossy().into_owned()
    };

    let mut entry = archive.by_name(&resolved_href)?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;

    let base_href = base_href_for(job_id, &resolved_href);
    let mime = mime_for(&resolved_href);
    let is_html = is_html_asset(&resolved_href);
    Ok(LoadedEntry {
        bytes,
        mime,
        base_href,
        resolved_href,
        is_html,
    })
}

fn base_href_for(job_id: &str, href: &str) -> String {
    let dir = Path::new(href)
        .parent()
        .map(|p| p.to_string_lossy().trim_end_matches('/').to_string())
        .unwrap_or_default();
    if dir.is_empty() {
        format!("epub://{}/", job_id)
    } else {
        format!("epub://{}/{}/", job_id, dir)
    }
}

fn split_job_id_and_href(
    uri: &tauri::http::Uri,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    if let Some(host) = uri.host() {
        let path = uri.path().trim_start_matches('/');
        if host != "localhost" && !path.is_empty() {
            return Ok((host.to_string(), path.to_string()));
        }

        let trimmed = path.strip_prefix(host).unwrap_or(path).trim_start_matches('/');
        if !trimmed.is_empty() {
            let (job_id, href) = trimmed.split_once('/').ok_or("missing path")?;
            return Ok((job_id.to_string(), href.to_string()));
        }
    }

    let uri_string = uri.to_string();
    let without_scheme = uri_string
        .strip_prefix("epub://")
        .ok_or("bad scheme")?;
    let without_localhost = without_scheme
        .strip_prefix("localhost/")
        .unwrap_or(without_scheme);
    let (job_id, href) = without_localhost
        .split_once('/')
        .ok_or("missing path")?;
    Ok((job_id.to_string(), href.to_string()))
}

fn is_html_asset(href: &str) -> bool {
    href.ends_with(".html") || href.ends_with(".xhtml") || href.ends_with(".htm")
}

fn decorate_html(html: &str, job_id: &str, base_href: &str, href: &str) -> String {
    let base_tag = format!(r#"<base href="{}">"#, base_href);
    let script = format!(
        r##"<script>
(function() {{
  const STYLE_ID = "xcalibre-reader-style";
  let spineIndex = 0;
  let totalSpineItems = 1;
  let pending = false;

  function ensureStyle(cssText) {{
    let style = document.getElementById(STYLE_ID);
    if (!style) {{
      style = document.createElement("style");
      style.id = STYLE_ID;
      document.head.appendChild(style);
    }}
    style.textContent = cssText || "";
  }}

  function applyCss(css) {{
    if (!css) return;
    const root = document.documentElement.style;
    Object.entries(css).forEach(([key, value]) => {{
      if (key === "--theme") {{
        document.body.classList.toggle("theme-sepia", value === "sepia");
        if (value === "dark") {{
          root.setProperty("--bg", "#1c1c1e");
          root.setProperty("--fg", "#e0e0e0");
        }} else if (value === "sepia") {{
          root.setProperty("--bg", "#f4ecd8");
          root.setProperty("--fg", "#433422");
        }} else {{
          root.setProperty("--bg", "#fefefe");
          root.setProperty("--fg", "#1a1a1a");
        }}
      }} else {{
        root.setProperty(key, String(value));
      }}
    }});
  }}

  function currentCfi() {{
    const elements = document.body.querySelectorAll("*");
    let elementIndex = 0;
    for (let i = 0; i < elements.length; i++) {{
      const rect = elements[i].getBoundingClientRect();
      if (rect.top >= 0) {{
        elementIndex = i;
        break;
      }}
    }}
    const scrollY = Math.round(window.scrollY);
    return `${{spineIndex}}:${{elementIndex}}:${{scrollY}}`;
  }}

  function calcProgress() {{
    const perItem = 100 / Math.max(1, totalSpineItems);
    const maxScroll = Math.max(1, document.documentElement.scrollHeight - window.innerHeight);
    const scrollRatio = Math.min(1, Math.max(0, window.scrollY / maxScroll));
    return Math.min(100, spineIndex * perItem + scrollRatio * perItem);
  }}

  function emitPosition() {{
    parent.postMessage({{
      type: "xcalibre-cfi",
      cfi: currentCfi(),
      progress: calcProgress(),
      jobId: {job_id:?},
      href: {href:?},
    }}, "*");
  }}

  function restoreCfi(cfi) {{
    if (typeof cfi !== "string") return;
    const parts = cfi.split(":");
    if (parts.length < 3) return;
    const incomingSpine = parseInt(parts[0], 10);
    const scrollY = parseInt(parts[2], 10);
    if (!Number.isNaN(incomingSpine)) spineIndex = incomingSpine;
    if (!Number.isNaN(scrollY)) window.scrollTo(0, scrollY);
  }}

  window.addEventListener("message", (event) => {{
    const data = event.data || {{}};
    if (data.type === "xcalibre-style") {{
      ensureStyle(data.css || "");
    }} else if (data.type === "xcalibre-css") {{
      applyCss(data.css || {{}});
    }} else if (data.type === "xcalibre-context") {{
      if (typeof data.spineIndex === "number") {{
        spineIndex = data.spineIndex;
      }}
      if (typeof data.totalSpineItems === "number" && data.totalSpineItems > 0) {{
        totalSpineItems = data.totalSpineItems;
      }}
      restoreCfi(data.cfi);
      emitPosition();
    }}
  }});

  window.addEventListener("scroll", () => {{
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => {{
      pending = false;
      emitPosition();
    }});
  }}, {{ passive: true }});

  window.addEventListener("load", emitPosition);
}})();
</script>"##,
    );

    if let Some(head_end) = html.find("</head>") {
        let mut injected = String::with_capacity(html.len() + base_tag.len() + script.len());
        injected.push_str(&html[..head_end]);
        injected.push_str(&base_tag);
        injected.push_str(&script);
        injected.push_str(&html[head_end..]);
        injected
    } else if let Some(body_end) = html.rfind("</body>") {
        let mut injected = String::with_capacity(html.len() + base_tag.len() + script.len());
        injected.push_str(&html[..body_end]);
        injected.push_str(&base_tag);
        injected.push_str(&script);
        injected.push_str(&html[body_end..]);
        injected
    } else {
        let mut injected = String::with_capacity(html.len() + base_tag.len() + script.len());
        injected.push_str(html);
        injected.push_str(&base_tag);
        injected.push_str(&script);
        injected
    }
}

fn mime_for(href: &str) -> &'static str {
    if href.ends_with(".html") || href.ends_with(".xhtml") || href.ends_with(".htm") {
        "text/html"
    } else if href.ends_with(".css") {
        "text/css"
    } else if href.ends_with(".png") {
        "image/png"
    } else if href.ends_with(".jpg") || href.ends_with(".jpeg") {
        "image/jpeg"
    } else if href.ends_with(".gif") {
        "image/gif"
    } else if href.ends_with(".svg") {
        "image/svg+xml"
    } else if href.ends_with(".xml") {
        "application/xml"
    } else {
        "application/octet-stream"
    }
}
