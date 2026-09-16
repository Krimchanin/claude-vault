import { useEffect, useMemo, useState } from "react";
import { Accounts } from "@/components/accounts";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion } from "motion/react";
import {
  ArchiveRestore,
  Check,
  ChevronRight,
  LoaderCircle,
  RefreshCw,
  Search,
  Settings as SettingsIcon,
  Vault,
} from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { Toaster } from "@/components/ui/sonner";

type Lang = "ru" | "en" | "zh" | "de" | "es" | "fr";
type Session = {
  id: string;
  cliSessionId?: string;
  title: string;
  turns: number | null;
  modifiedAt: number;
  archived: boolean;
  size: number;
};
type Message = {
  role: "user" | "assistant";
  content: string;
  timestamp?: string;
};
type Scan = { sessions: Session[]; sourcePath: string; archivePath: string };
type Settings = {
  claudeRoot: string;
  transcriptRoot: string;
  language: string;
};
const ease = [0.22, 1, 0.36, 1] as const;
const languages: Record<Lang, string> = {
  ru: "Русский",
  en: "English",
  zh: "中文",
  de: "Deutsch",
  es: "Español",
  fr: "Français",
};
const copy = {
  ru: {
    found: "сессий",
    saved: "сохранено",
    loading: "Ищем локальные сессии…",
    sync: "Дополнить из кэша",
    restore: "Вернуть отсутствующие",
    chats: "Диалоги",
    hint: "Нажмите на диалог, чтобы открыть транскрипт",
    search: "Поиск",
    turns: "ходов",
    inVault: "Сохранён",
    inCache: "В кэше",
    empty: "Ничего не найдено",
    noText: "Текст этого диалога не найден",
    you: "Вы",
    claude: "Claude",
    settings: "Настройки",
    settingsHint: "Пути к локальным данным Claude и язык интерфейса",
    language: "Язык",
    claudePath: "Папка данных Claude App",
    transcriptPath: "Папка транскриптов Claude Code",
    save: "Сохранить",
    archiveUpdated: "Архив обновлён",
    restored: "Восстановление завершено",
    local: "локальный транскрипт",
  },
  en: {
    found: "sessions",
    saved: "saved",
    loading: "Finding local sessions…",
    sync: "Add from cache",
    restore: "Restore missing",
    chats: "Conversations",
    hint: "Select a conversation to open its transcript",
    search: "Search",
    turns: "turns",
    inVault: "Saved",
    inCache: "In cache",
    empty: "Nothing found",
    noText: "Transcript not found",
    you: "You",
    claude: "Claude",
    settings: "Settings",
    settingsHint: "Claude data locations and interface language",
    language: "Language",
    claudePath: "Claude App data folder",
    transcriptPath: "Claude Code transcripts folder",
    save: "Save",
    archiveUpdated: "Archive updated",
    restored: "Restore complete",
    local: "local transcript",
  },
  zh: {
    found: "个会话",
    saved: "已保存",
    loading: "正在查找本地会话…",
    sync: "从缓存补充",
    restore: "恢复缺失数据",
    chats: "对话",
    hint: "点击对话查看完整记录",
    search: "搜索",
    turns: "轮",
    inVault: "已保存",
    inCache: "缓存中",
    empty: "未找到内容",
    noText: "未找到对话记录",
    you: "你",
    claude: "Claude",
    settings: "设置",
    settingsHint: "Claude 数据路径与界面语言",
    language: "语言",
    claudePath: "Claude App 数据文件夹",
    transcriptPath: "Claude Code 记录文件夹",
    save: "保存",
    archiveUpdated: "存档已更新",
    restored: "恢复完成",
    local: "本地对话记录",
  },
  de: {
    found: "Sitzungen",
    saved: "gespeichert",
    loading: "Lokale Sitzungen werden gesucht…",
    sync: "Aus Cache ergänzen",
    restore: "Fehlende wiederherstellen",
    chats: "Unterhaltungen",
    hint: "Unterhaltung öffnen, um das Transkript zu sehen",
    search: "Suchen",
    turns: "Runden",
    inVault: "Gespeichert",
    inCache: "Im Cache",
    empty: "Nichts gefunden",
    noText: "Kein Transkript gefunden",
    you: "Du",
    claude: "Claude",
    settings: "Einstellungen",
    settingsHint: "Claude-Datenpfade und Sprache",
    language: "Sprache",
    claudePath: "Claude App-Datenordner",
    transcriptPath: "Claude Code-Transkriptordner",
    save: "Speichern",
    archiveUpdated: "Archiv aktualisiert",
    restored: "Wiederherstellung abgeschlossen",
    local: "lokales Transkript",
  },
  es: {
    found: "sesiones",
    saved: "guardadas",
    loading: "Buscando sesiones locales…",
    sync: "Añadir desde caché",
    restore: "Restaurar faltantes",
    chats: "Conversaciones",
    hint: "Selecciona una conversación para ver su contenido",
    search: "Buscar",
    turns: "turnos",
    inVault: "Guardado",
    inCache: "En caché",
    empty: "No se encontraron resultados",
    noText: "No se encontró la transcripción",
    you: "Tú",
    claude: "Claude",
    settings: "Ajustes",
    settingsHint: "Rutas de datos de Claude e idioma",
    language: "Idioma",
    claudePath: "Carpeta de datos de Claude App",
    transcriptPath: "Carpeta de transcripciones",
    save: "Guardar",
    archiveUpdated: "Archivo actualizado",
    restored: "Restauración completada",
    local: "transcripción local",
  },
  fr: {
    found: "sessions",
    saved: "sauvegardées",
    loading: "Recherche des sessions locales…",
    sync: "Ajouter depuis le cache",
    restore: "Restaurer les éléments manquants",
    chats: "Conversations",
    hint: "Sélectionnez une conversation pour lire la transcription",
    search: "Rechercher",
    turns: "tours",
    inVault: "Sauvegardé",
    inCache: "En cache",
    empty: "Aucun résultat",
    noText: "Transcription introuvable",
    you: "Vous",
    claude: "Claude",
    settings: "Réglages",
    settingsHint: "Emplacements des données Claude et langue",
    language: "Langue",
    claudePath: "Dossier de données Claude App",
    transcriptPath: "Dossier des transcriptions",
    save: "Enregistrer",
    archiveUpdated: "Archive mise à jour",
    restored: "Restauration terminée",
    local: "transcription locale",
  },
};

function Loading() {
  return (
    <div className="space-y-2 pt-2">
      {[0, 1, 2, 3, 4].map((i) => (
        <motion.div
          key={i}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: i * 0.06, duration: 0.45, ease }}
          className="flex items-center gap-3 rounded-xl border bg-card p-4"
        >
          <Skeleton className="size-9 rounded-lg" />
          <div className="flex-1 space-y-2">
            <Skeleton className="h-3.5 w-52" />
            <Skeleton className="h-3 w-28" />
          </div>
          <Skeleton className="h-5 w-16 rounded-full" />
        </motion.div>
      ))}
    </div>
  );
}

export default function App() {
  const [data, setData] = useState<Scan>({
      sessions: [],
      sourcePath: "",
      archivePath: "",
    }),
    [loading, setLoading] = useState(true),
    [busy, setBusy] = useState<"sync" | "restore" | null>(null),
    [query, setQuery] = useState(""),
    [selected, setSelected] = useState<Session | null>(null),
    [messages, setMessages] = useState<Message[]>([]),
    [dialogLoading, setDialogLoading] = useState(false),
    [settingsOpen, setSettingsOpen] = useState(false),
    [settings, setSettings] = useState<Settings>({
      claudeRoot: "",
      transcriptRoot: "",
      language: "ru",
    });
  const lang = (settings.language in copy ? settings.language : "ru") as Lang,
    t = copy[lang];
  const scan = async () => {
    setLoading(true);
    try {
      setData(await invoke<Scan>("scan_sessions"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => {
    invoke<Settings>("get_settings").then(setSettings).finally(scan);
  }, []);
  const run = async (kind: "sync" | "restore") => {
    setBusy(kind);
    try {
      const r =
        kind === "sync"
          ? await invoke<{ copied: number; updated: number }>("sync_cache")
          : await invoke<{ restored: number; skipped: number }>(
              "restore_missing",
            );
      toast.success(kind === "sync" ? t.archiveUpdated : t.restored, {
        description:
          kind === "sync"
            ? `+${"copied" in r ? r.copied : 0}`
            : `+${"restored" in r ? r.restored : 0}`,
      });
      await scan();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(null);
    }
  };
  const open = async (s: Session) => {
    setSelected(s);
    setMessages([]);
    setDialogLoading(true);
    try {
      setMessages(
        s.cliSessionId
          ? await invoke<Message[]>("load_transcript", {
              cliSessionId: s.cliSessionId,
            })
          : [],
      );
    } catch (e) {
      toast.error(String(e));
    } finally {
      setDialogLoading(false);
    }
  };
  const save = async () => {
    try {
      await invoke("save_settings", { settings });
      setSettingsOpen(false);
      toast.success(t.save);
      await scan();
    } catch (e) {
      toast.error(String(e));
    }
  };
  const sessions = useMemo(
      () =>
        data.sessions.filter((s) =>
          s.title.toLowerCase().includes(query.toLowerCase()),
        ),
      [data.sessions, query],
    ),
    saved = data.sessions.filter((s) => s.archived).length;
  return (
    <div className="dark min-h-screen bg-background text-foreground">
      <motion.main
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.55 }}
        className="mx-auto max-w-4xl px-6 py-8 sm:px-8"
      >
        <header className="mb-8 flex items-start justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="grid size-10 place-items-center rounded-xl border bg-card shadow-sm">
              <Vault className="size-5" />
            </div>
            <div>
              <h1 className="text-[20px] font-semibold tracking-tight">
                Claude Vault
              </h1>
              <p className="text-muted-foreground text-[13px]">
                {loading
                  ? t.loading
                  : `${data.sessions.length} ${t.found} · ${saved} ${t.saved}`}
              </p>
            </div>
          </div>
          <div className="flex">
            <Accounts language={lang} />
            <Button
              variant="ghost"
              size="icon"
              onClick={scan}
              disabled={loading}
            >
              <RefreshCw
                className={`size-4 ${loading ? "animate-spin" : ""}`}
              />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSettingsOpen(true)}
            >
              <SettingsIcon className="size-4" />
            </Button>
          </div>
        </header>
        <section className="mb-7 flex flex-wrap gap-2">
          <Button onClick={() => run("sync")} disabled={!!busy || loading}>
            {busy === "sync" ? (
              <LoaderCircle className="animate-spin" />
            ) : (
              <Check />
            )}
            {t.sync}
          </Button>
          <Button
            variant="outline"
            onClick={() => run("restore")}
            disabled={!!busy || loading}
          >
            {busy === "restore" ? (
              <LoaderCircle className="animate-spin" />
            ) : (
              <ArchiveRestore />
            )}
            {t.restore}
          </Button>
        </section>
        <Separator />
        <section className="pt-6">
          <div className="mb-4 flex items-center justify-between gap-4">
            <div>
              <h2 className="text-[15px] font-medium">{t.chats}</h2>
              <p className="text-muted-foreground mt-0.5 text-xs">{t.hint}</p>
            </div>
            <div className="relative w-56">
              <Search className="text-muted-foreground absolute left-3 top-2.5 size-4" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t.search}
                className="pl-9"
              />
            </div>
          </div>
          {loading ? (
            <Loading />
          ) : (
            <motion.div layout className="space-y-2">
              <AnimatePresence initial={false}>
                {sessions.map((s, i) => (
                  <motion.button
                    layout
                    key={s.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.98 }}
                    transition={{
                      delay: Math.min(i * 0.025, 0.25),
                      duration: 0.4,
                      ease,
                    }}
                    onClick={() => open(s)}
                    className="group flex w-full items-center gap-4 rounded-xl border bg-card px-4 py-3.5 text-left shadow-sm transition-colors hover:bg-accent/60"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-medium">
                        {s.title}
                      </div>
                      <div className="text-muted-foreground mt-1 text-[11px]">
                        {s.turns ?? "—"} {t.turns} ·{" "}
                        {new Date(s.modifiedAt * 1000).toLocaleDateString(
                          lang,
                          {
                            day: "numeric",
                            month: "short",
                            hour: "2-digit",
                            minute: "2-digit",
                          },
                        )}
                      </div>
                    </div>
                    <Badge variant={s.archived ? "secondary" : "outline"}>
                      {s.archived ? t.inVault : t.inCache}
                    </Badge>
                    <ChevronRight className="text-muted-foreground size-4 transition-transform duration-300 group-hover:translate-x-0.5" />
                  </motion.button>
                ))}
              </AnimatePresence>
              {!sessions.length && (
                <div className="text-muted-foreground py-16 text-center text-sm">
                  {t.empty}
                </div>
              )}
            </motion.div>
          )}
        </section>
      </motion.main>
      <Sheet open={!!selected} onOpenChange={(o) => !o && setSelected(null)}>
        <SheetContent className="w-full border-l p-0 sm:max-w-2xl">
          <SheetHeader className="border-b p-6">
            <SheetTitle>{selected?.title}</SheetTitle>
            <SheetDescription>
              {selected?.turns ?? "—"} {t.turns} · {t.local}
            </SheetDescription>
          </SheetHeader>
          <ScrollArea className="h-[calc(100vh-93px)]">
            <div className="space-y-6 p-6">
              {dialogLoading ? (
                <Loading />
              ) : messages.length ? (
                messages.map((m, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{
                      delay: Math.min(i * 0.035, 0.5),
                      duration: 0.45,
                      ease,
                    }}
                    className={m.role === "user" ? "ml-10" : "mr-10"}
                  >
                    <div className="text-muted-foreground mb-1.5 text-[11px]">
                      {m.role === "user" ? t.you : t.claude}
                    </div>
                    <div
                      className={
                        m.role === "user"
                          ? "whitespace-pre-wrap rounded-2xl bg-secondary px-4 py-3 text-[13px] leading-relaxed"
                          : "whitespace-pre-wrap text-[13px] leading-relaxed"
                      }
                    >
                      {m.content}
                    </div>
                  </motion.div>
                ))
              ) : (
                <div className="text-muted-foreground py-20 text-center text-sm">
                  {t.noText}
                </div>
              )}
            </div>
          </ScrollArea>
        </SheetContent>
      </Sheet>
      <Sheet open={settingsOpen} onOpenChange={setSettingsOpen}>
        <SheetContent className="sm:max-w-md">
          <SheetHeader>
            <SheetTitle>{t.settings}</SheetTitle>
            <SheetDescription>{t.settingsHint}</SheetDescription>
          </SheetHeader>
          <div className="space-y-6 px-4">
            <div className="space-y-2">
              <Label>{t.language}</Label>
              <Select
                value={lang}
                onValueChange={(v) => setSettings({ ...settings, language: v })}
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(languages).map(([v, n]) => (
                    <SelectItem key={v} value={v}>
                      {n}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>{t.claudePath}</Label>
              <Input
                value={settings.claudeRoot}
                onChange={(e) =>
                  setSettings({ ...settings, claudeRoot: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label>{t.transcriptPath}</Label>
              <Input
                value={settings.transcriptRoot}
                onChange={(e) =>
                  setSettings({ ...settings, transcriptRoot: e.target.value })
                }
              />
            </div>
            <Button className="w-full" onClick={save}>
              {t.save}
            </Button>
          </div>
        </SheetContent>
      </Sheet>
      <Toaster position="bottom-right" />
    </div>
  );
}
