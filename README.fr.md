# Claude Vault

[English](README.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · **Français**

**Changez de compte Claude sans perdre l'historique.** Claude Vault est un outil de bureau non officiel et local permettant d'afficher, sauvegarder et restaurer les données de Claude Desktop et Claude Code directement depuis le système de fichiers.

Claude Desktop peut retirer les conversations locales visibles lors d'un changement de compte. Claude Vault conserve les métadonnées, les transcriptions JSONL complètes, les pièces jointes et les fichiers de travail dans une archive indépendante. La restauration ajoute uniquement les fichiers manquants et n'écrase jamais les données Claude existantes.

Aucun export officiel, aucun compte et aucun envoi de données. Windows, macOS et Linux sont pris en charge avec des chemins configurables. Développement : `npm install`, puis `npm run tauri dev`. Voir [docs/archive-format.md](docs/archive-format.md).

Projet indépendant, sans affiliation avec Anthropic. Licence MIT.
