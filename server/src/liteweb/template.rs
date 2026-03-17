pub fn wrap(title: &str, byline: Option<&str>, content: &str) -> String {
    let byline_html = if let Some(by) = byline {
        format!("<div class=\"by\">{}</div>", html_esc(by))
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{max-width:680px;margin:0 auto;padding:16px;font:18px/1.6 system-ui,-apple-system,sans-serif;color:#222;background:#fff;word-wrap:break-word}}
h1{{font-size:1.5em;line-height:1.25;margin:0 0 8px}}
h2,h3,h4{{margin:20px 0 8px}}
p{{margin:0 0 12px}}
.by{{color:#666;font-size:.85em;margin:0 0 20px}}
img{{max-width:100%;height:auto;border-radius:4px;margin:12px 0}}
a{{color:#1a73e8;text-decoration:none}}
a:hover{{text-decoration:underline}}
blockquote{{border-left:3px solid #ccc;margin:16px 0;padding:0 16px;color:#555}}
pre{{background:#f5f5f5;border-radius:4px;padding:12px;overflow-x:auto;font-size:.9em;margin:12px 0}}
code{{background:#f5f5f5;border-radius:3px;padding:2px 4px;font-size:.9em}}
pre code{{background:none;padding:0}}
table{{border-collapse:collapse;width:100%;margin:12px 0}}
td,th{{border:1px solid #ddd;padding:8px;text-align:left}}
th{{background:#f9f9f9}}
hr{{border:none;border-top:1px solid #eee;margin:20px 0}}
li{{margin:4px 0}}
ul,ol{{padding-left:24px;margin:8px 0}}
figure{{margin:12px 0}}
figcaption{{color:#666;font-size:.85em;margin-top:4px}}
.ft{{color:#999;font-size:.75em;margin-top:32px;padding-top:12px;border-top:1px solid #eee}}
@media(prefers-color-scheme:dark){{body{{background:#1a1a1a;color:#e0e0e0}}a{{color:#8ab4f8}}blockquote{{border-color:#555;color:#aaa}}pre,code{{background:#2a2a2a}}td,th{{border-color:#444}}th{{background:#252525}}.by,.ft{{color:#888}}hr{{border-color:#333}}}}
</style>
</head>
<body>
<h1>{title}</h1>
{byline_html}
{content}
<p class="ft">Optimized by RedBrowser LiteWeb</p>
</body>
</html>"#,
        title = html_esc(title),
        byline_html = byline_html,
        content = content,
    )
}

fn html_esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
