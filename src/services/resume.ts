import { invoke } from "@tauri-apps/api/core";
import { RESUME_CONFIGS, type Session } from "../types";

function extractSessionId(sourcePath: string, agent: string): string | null {
  if (agent === "claude-code") {
    const direct = sourcePath.match(/([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})\.jsonl$/i);
    if (direct) return direct[1];
    const anyUuid = sourcePath.match(/([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})/i);
    return anyUuid ? anyUuid[1] : null;
  }
  if (agent === "codex") {
    const rollout = sourcePath.match(/([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})\.jsonl$/i);
    if (rollout) return rollout[1];
    const legacy = sourcePath.match(/sessions\/([a-f0-9-]{36})(?:\/|$)/i);
    return legacy ? legacy[1] : null;
  }
  return null;
}

async function resolveSessionId(session: Session): Promise<string | null> {
  const extracted = extractSessionId(session.sourcePath, session.agent);
  if (extracted) return extracted;

  if (session.agent !== "gemini") return null;

  try {
    const resolved = await invoke<string>("resolve_gemini_resume_target", {
      sourcePath: session.sourcePath,
      workspace: session.workspace?.trim() || null,
    });
    const cleaned = typeof resolved === "string" ? resolved.trim() : "";
    return cleaned || null;
  } catch (err) {
    console.warn("Gemini resume target resolution failed:", err);
    return null;
  }
}

function escapeAppleScript(s: string): string {
  return s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function shellQuote(value: string): string {
  return `'${value.replace(/'/g, "'\"'\"'")}'`;
}

function isGeminiHashWorkspace(path: string | null | undefined): boolean {
  if (!path) return false;
  return /\/\.gemini\/tmp\/[a-f0-9]{64}(?:\/|$)/i.test(path.trim());
}

async function resolveWorkspace(session: Session): Promise<string | null> {
  const workspace = session.workspace?.trim() || null;

  if (session.agent === "claude-code") {
    if (workspace) return workspace;

    try {
      const resolved = await invoke<string>("resolve_claude_workspace", {
        sourcePath: session.sourcePath,
        workspace,
      });
      const cleaned = typeof resolved === "string" ? resolved.trim() : "";
      if (cleaned) return cleaned;
    } catch (err) {
      console.warn("Claude workspace resolution failed:", err);
    }

    return null;
  }

  if (session.agent !== "gemini") {
    return workspace;
  }

  if (workspace && !isGeminiHashWorkspace(workspace)) {
    return workspace;
  }

  try {
    const resolved = await invoke<string>("resolve_gemini_workspace", {
      sourcePath: session.sourcePath,
      workspace,
    });
    const cleaned = typeof resolved === "string" ? resolved.trim() : "";
    if (cleaned) return cleaned;
  } catch (err) {
    console.warn("Gemini workspace resolution failed:", err);
  }

  return workspace && !isGeminiHashWorkspace(workspace) ? workspace : null;
}

export async function buildResumeCommand(session: Session): Promise<string | null> {
  const config = RESUME_CONFIGS[session.agent];
  if (!config) return null;

  const sessionId = await resolveSessionId(session);
  if (!sessionId && config.resumeArgs.includes("{sessionId}")) return null;

  const args = config.resumeArgs.map((arg) => arg.replace("{sessionId}", sessionId || ""));
  return `${config.command} ${args.join(" ")}`;
}

export async function buildAppleScript(session: Session): Promise<string | null> {
  const resumeCmd = await buildResumeCommand(session);
  if (!resumeCmd) return null;

  const workspace = await resolveWorkspace(session);
  const launchCmd = workspace ? `cd -- ${shellQuote(workspace)} && ${resumeCmd}` : resumeCmd;

  let script = 'tell application "iTerm2"\n  activate\n  create window with default profile\n';
  script += `  tell current session of current window to write text "${escapeAppleScript(launchCmd)}"\n`;
  script += "end tell";
  return script;
}

export async function resumeSession(session: Session): Promise<boolean> {
  const script = await buildAppleScript(session);
  if (!script) {
    console.error("Cannot build resume command for", session.agent);
    return false;
  }

  try {
    await invoke<string>("run_osascript", { script });
    return true;
  } catch (err) {
    console.error("Resume error:", err);
    return false;
  }
}
