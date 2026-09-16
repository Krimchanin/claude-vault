# Claude Vault

[English](README.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · **Deutsch** · [Español](README.es.md) · [Français](README.fr.md)

**Claude-Konten wechseln, ohne den Chatverlauf zu verlieren.** Claude Vault ist ein inoffizielles, lokal arbeitendes Desktop-Werkzeug zum Anzeigen, Sichern und Wiederherstellen von Claude-Desktop- und Claude-Code-Daten direkt aus dem Dateisystem.

Claude Desktop kann beim Kontowechsel lokal sichtbare Chats entfernen. Claude Vault bewahrt Metadaten, vollständige JSONL-Verläufe, Anhänge und Arbeitsdateien in einem unabhängigen Archiv auf. Beim Wiederherstellen werden nur fehlende Dateien ergänzt; vorhandene Claude-Dateien werden niemals überschrieben.

Kein offizieller Export, kein Konto und kein Upload. Windows, macOS und Linux werden unterstützt; Quellpfade sind konfigurierbar. Entwicklung: `npm install` und `npm run tauri dev`. Archivdetails stehen in [docs/archive-format.md](docs/archive-format.md).

Unabhängiges Projekt ohne Verbindung zu Anthropic. MIT-Lizenz.
