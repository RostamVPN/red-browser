# RedBrowser

Censorship-resistant browser with built-in Instagram, Telegram, and WhatsApp — all tunneled through DNS queries.

## How it works

RedBrowser is not a VPN. It's a browser where the DNS tunnel IS the internet connection.

```
RedBrowser App (your phone)              RedBrowser Server (Frankfurt)

  You type a URL or                      Server fetches the real page
  open Instagram feed                    (2.4 MB of JS, ads, trackers)
       |                                        |
       | DNS queries                            v
       | (looks like normal DNS)         Strips everything:
       |                                 - JavaScript
       +=============================>  - Ads and trackers
       <=============================+  - Web bugs and pixels
       |                                 - Fonts and bloat
       | Clean content                          |
       | (15 KB instead of 2.4 MB)              v
       v                                 Returns clean, fast HTML
  Renders instantly                      with optimized images
```

**What the user sees**: A fast browser. Pages load in 2-3 seconds.

**What the censor sees**: Normal DNS TXT queries. No VPN. No HTTPS to blocked sites.

**What the ISP sees**: Nothing suspicious. DNS traffic is ubiquitous.

## Features

### Web Browsing (LiteWeb)
- Server-side rendering strips ads, trackers, scripts, web bugs
- 100x bandwidth reduction (2.4 MB page → 15 KB)
- Images auto-optimized to WebP thumbnails
- Dark mode support
- Built-in DuckDuckGo search

### Instagram Lite *(coming soon)*
- View feed, stories, profiles
- Like and comment
- Direct messages
- Search users and tags
- No Reels video (bandwidth) — thumbnails only

### Telegram Lite *(coming soon)*
- Chat list and conversations
- Send and receive text messages
- Photo messages (thumbnails, tap for full)
- Channels and groups
- Powered by TDLib

### WhatsApp Lite *(coming soon)*
- Link via QR code
- Chat list and conversations
- Send and receive text messages
- Photo messages
- Powered by WhatsApp Web bridge

## Architecture

```
red-browser/
├── protocol/    # Shared wire protocol (CBOR over smux streams)
├── server/      # Content proxy server
│   └── liteweb/ # Web rendering engine (fetch → extract → sanitize → optimize)
├── android/     # Android app (Kotlin + Compose) — coming soon
└── ios/         # iOS app (Swift + SwiftUI) — coming soon
```

### Protocol

Binary CBOR messages over length-prefixed frames:

```
Client → Server:
  Browse { url }              Fetch and render a web page
  Search { query }            DuckDuckGo search
  IgFeed { cursor }           Instagram feed
  TgGetChats { offset }       Telegram chat list
  WaGetChats                  WhatsApp chat list
  Ping { ts }                 Latency check

Server → Client:
  Page { html, title }        Rendered page (gzip compressed)
  SearchResults { html }      Search results
  IgFeedResult { posts }      Instagram posts with thumbnails
  TgChatsResult { chats }     Telegram chats
  Error { code, message }     Error response
```

### LiteWeb Pipeline

```
URL → Fetch (Chrome UA, 15s timeout)
    → Readability extraction (title, byline, main content)
    → Sanitize (strip scripts, ads, trackers, event handlers, data-* attrs)
    → Image optimization (resize, WebP, inline as data URIs)
    → Template (minimal HTML, ~500 bytes CSS, dark mode)
    → Gzip compress
    → Return through DNS tunnel
```

### Tracker/Ad Blocking

100+ tracker domains blocked at the server level:
- Google Analytics, Tag Manager, Ads
- Facebook Pixel, Meta tracking
- Twitter/X analytics
- Adobe, Yandex, Hotjar, Segment, Mixpanel
- All major ad networks (DoubleClick, AdRoll, Criteo, Taboola, Outbrain)
- Cookie consent popups (OneTrust, Cookiebot)
- Chat widgets (Intercom, Drift, Crisp)

Plus URL tracking parameter stripping (utm_*, fbclid, gclid, etc.)

## Building

```bash
# Server
cargo build --release -p red-server
# Binary: target/release/red-server

# Run
./target/release/red-server --listen 127.0.0.1:8400

# Test with curl (length-prefixed CBOR — use the test client)
cargo test
```

### Cross-compilation (Linux server)

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release -p red-server --target x86_64-unknown-linux-musl
```

## Bandwidth Budget

| Action | Original | RedBrowser | Time @ 50KB/s |
|--------|----------|------------|---------------|
| News article | 2.4 MB | 15-50 KB | 1-2s |
| Instagram feed (20 posts) | 8 MB | 80 KB | 2s |
| Telegram chat list | N/A | 5 KB | 0.1s |
| Send text message | N/A | 0.2 KB | instant |
| DuckDuckGo search | 500 KB | 10 KB | 0.2s |

## Security

- No JavaScript ever executes on the client
- No tracking scripts, pixels, or beacons
- No fingerprinting (canvas, WebRTC, font enumeration)
- All content sanitized server-side before delivery
- DNS tunnel encrypted with Noise_NK (forward secrecy)
- Server identity verified via pre-shared public key

## DNS Tunnel

RedBrowser uses [Nooshdaroo](https://github.com/RostamVPN/nooshdaroo) as the transport layer — a Rust DNS tunnel that encodes traffic in DNS TXT queries, indistinguishable from normal DNS lookups.

## License

MIT

Copyright (c) 2026 Internet Mastering & Company, Inc.
