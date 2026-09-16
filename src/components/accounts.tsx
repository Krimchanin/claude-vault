import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LoaderCircle, Users } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";

type Profile = { id: string; name: string };
type State = { active: string | null; recovery_required: boolean };
const copy: Record<string, readonly string[]> = {
  ru: [
    "Аккаунты",
    "Закройте Claude перед сохранением и переключением. Первый аккаунт — текущий вход; новый аккаунт потребует войти один раз.",
    "Название аккаунта",
    "Сохранить текущий",
    "Добавить новый",
    "Переключить",
    "Активен",
    "Открыть Claude",
    "Полностью закройте Claude Desktop, включая значок в трее, и повторите нажатие.",
    "Готово",
    "Восстановить предыдущий профиль",
    "Данные входа хранятся только на этом компьютере отдельно от архива. Не экспортируйте папку account-profiles. Функция пока экспериментальная, только Windows. CLI и системные хранилища не переключаются.",
    "Путь к Claude (необязательно)",
    "Требуется откат незавершённого переключения.",
  ],
  en: [
    "Accounts",
    "Close Claude before saving or switching. Save your current login first; sign in once when opening a new account.",
    "Account name",
    "Save current",
    "Add new",
    "Switch",
    "Active",
    "Open Claude",
    "Fully quit Claude Desktop, including its tray icon, then try again.",
    "Done",
    "Recover previous profile",
    "Login data stays on this computer, separate from chat backups. Never export account-profiles. Experimental, Windows only. CLI and system credential stores are not switched.",
    "Claude executable (optional)",
    "Recover the interrupted switch before continuing.",
  ],
  zh: [
    "账户",
    "保存或切换前请退出 Claude。先保存当前账户，新账户首次需要登录。",
    "账户名称",
    "保存当前",
    "添加",
    "切换",
    "当前",
    "打开 Claude",
    "请完全退出 Claude（包括托盘），然后重试。",
    "完成",
    "恢复上一配置",
    "登录数据仅保存在本机，与聊天备份分离。不要导出 account-profiles。实验功能，仅支持 Windows。CLI 和系统凭据不切换。",
    "Claude 路径（可选）",
    "请先恢复未完成的切换。",
  ],
  de: [
    "Konten",
    "Claude vor dem Speichern oder Wechseln beenden. Zuerst das aktuelle Konto sichern; neue Konten erfordern einmalige Anmeldung.",
    "Kontoname",
    "Aktuelles sichern",
    "Hinzufügen",
    "Wechseln",
    "Aktiv",
    "Claude öffnen",
    "Claude einschließlich Taskleistensymbol vollständig beenden und erneut versuchen.",
    "Fertig",
    "Vorheriges Profil retten",
    "Anmeldedaten bleiben lokal und getrennt vom Archiv. account-profiles niemals exportieren. Experimentell, nur Windows. CLI und System-Zugangsdaten bleiben unverändert.",
    "Claude-Pfad (optional)",
    "Unvollständigen Wechsel zuerst wiederherstellen.",
  ],
  es: [
    "Cuentas",
    "Cierra Claude antes de guardar o cambiar. Guarda primero la cuenta actual; inicia sesión una vez en cada cuenta nueva.",
    "Nombre",
    "Guardar actual",
    "Añadir",
    "Cambiar",
    "Activa",
    "Abrir Claude",
    "Cierra Claude por completo, incluida la bandeja, y vuelve a intentarlo.",
    "Hecho",
    "Recuperar perfil anterior",
    "Las sesiones quedan locales, separadas del archivo. No exportes account-profiles. Experimental, solo Windows. CLI y credenciales del sistema no cambian.",
    "Ruta de Claude (opcional)",
    "Recupera primero el cambio interrumpido.",
  ],
  fr: [
    "Comptes",
    "Fermez Claude avant de sauvegarder ou changer. Sauvegardez d'abord le compte actuel; connectez-vous une fois par nouveau compte.",
    "Nom",
    "Sauvegarder actuel",
    "Ajouter",
    "Changer",
    "Actif",
    "Ouvrir Claude",
    "Quittez complètement Claude, y compris la zone de notification, puis réessayez.",
    "Terminé",
    "Récupérer le profil précédent",
    "Les sessions restent locales et séparées des archives. N'exportez jamais account-profiles. Expérimental, Windows uniquement. CLI et identifiants système ne changent pas.",
    "Chemin Claude (facultatif)",
    "Récupérez d'abord le changement interrompu.",
  ],
};

export function Accounts({ language }: { language: string }) {
  const t = copy[language] ?? copy.en;
  const [open, setOpen] = useState(false);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [executable, setExecutable] = useState("");
  const [busy, setBusy] = useState(false);
  const [warning, setWarning] = useState("");
  const [recovery, setRecovery] = useState(false);
  async function refresh() {
    const [items, state] = await Promise.all([
      invoke<Profile[]>("list_profiles"),
      invoke<State>("get_account_state"),
    ]);
    setProfiles(items);
    setActive(state.active);
    setRecovery(state.recovery_required);
    setWarning(state.recovery_required ? t[13] : "");
  }
  async function perform(command: string, args?: Record<string, unknown>) {
    setBusy(true);
    setWarning("");
    try {
      await invoke(command, args);
      await refresh();
      if (command === "create_profile") setName("");
      toast.success(t[9]);
    } catch (error) {
      const value = String(error);
      if (value === "CLAUDE_RUNNING") setWarning(t[8]);
      else if (value === "RECOVERY_REQUIRED") {
        setRecovery(true);
        setWarning(t[13]);
      } else toast.error(value);
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    if (open) {
      setBusy(true);
      refresh()
        .catch((e) => toast.error(String(e)))
        .finally(() => setBusy(false));
    }
  }, [open]);
  return (
    <>
      <Button
        variant="ghost"
        size="icon"
        aria-label={t[0]}
        title={t[0]}
        onClick={() => setOpen(true)}
      >
        <Users className="size-4" />
      </Button>
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent className="sm:max-w-md">
          <SheetHeader>
            <SheetTitle>{t[0]}</SheetTitle>
            <SheetDescription>{t[1]}</SheetDescription>
          </SheetHeader>
          <div className="space-y-3 px-4">
            <Input
              aria-label={t[2]}
              placeholder={t[2]}
              maxLength={40}
              value={name}
              disabled={busy}
              onChange={(e) => setName(e.target.value)}
            />
            <Button
              className="w-full"
              disabled={busy || recovery || !name.trim()}
              onClick={() =>
                void perform("create_profile", { name, capture: !active })
              }
            >
              {busy && <LoaderCircle className="animate-spin" />}
              {active ? t[4] : t[3]}
            </Button>
            {warning && (
              <p
                role="alert"
                className="rounded-lg border p-3 text-sm leading-relaxed"
              >
                {warning}
              </p>
            )}
            {recovery && (
              <Button
                variant="outline"
                disabled={busy}
                className="w-full"
                onClick={() => void perform("recover_account")}
              >
                {t[10]}
              </Button>
            )}
          </div>
          <ScrollArea className="min-h-0 flex-1 px-4">
            <div className="space-y-2 py-2">
              {profiles.map((profile) => (
                <div
                  key={profile.id}
                  className="flex items-center justify-between gap-3 rounded-xl border bg-card p-3"
                >
                  <span className="truncate text-sm">{profile.name}</span>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busy || recovery || active === profile.id}
                    onClick={() =>
                      void perform("switch_profile", { id: profile.id })
                    }
                  >
                    {active === profile.id ? t[6] : t[5]}
                  </Button>
                </div>
              ))}
            </div>
          </ScrollArea>
          <div className="space-y-3 px-4 pb-5">
            <Input
              aria-label={t[12]}
              placeholder={t[12]}
              value={executable}
              onChange={(e) => setExecutable(e.target.value)}
            />
            <Button
              variant="outline"
              className="w-full"
              disabled={busy || recovery}
              onClick={() => void perform("open_claude", { executable })}
            >
              {t[7]}
            </Button>
            <p className="text-xs leading-relaxed text-muted-foreground">
              {t[11]}
            </p>
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}
