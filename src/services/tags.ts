import { invoke } from "@tauri-apps/api/core";
import type { Session, SessionMessage, SessionTag, SessionTagStore } from "../types";

const tagCache: Record<string, SessionTag> = {};

function truncateChars(text: string, limit: number): string {
  return text.slice(0, limit);
}

function normalizeTag(raw: Record<string, unknown>, keyFallback?: string): SessionTag {
  const sessionId =
    (typeof raw.sessionId === "string" && raw.sessionId) ||
    (typeof raw.session_id === "string" && raw.session_id) ||
    keyFallback ||
    "";
  const summary = typeof raw.summary === "string" && raw.summary.trim() ? raw.summary.trim() : "未分类";
  const category = typeof raw.category === "string" && raw.category.trim() ? raw.category.trim() : "misc";
  const taggedAtRaw = raw.taggedAt ?? raw.tagged_at;
  const taggedAt = typeof taggedAtRaw === "number" ? taggedAtRaw : Date.now();
  const model = typeof raw.model === "string" && raw.model.trim() ? raw.model.trim() : undefined;

  return { sessionId, summary, category, taggedAt, model };
}

function sessionTitle(session: Session): string {
  if (session.title?.trim()) return session.title.trim();
  if (session.snippet?.trim()) return session.snippet.trim();
  const parts = session.sourcePath.split("/");
  return parts[parts.length - 1] || session.id;
}

function buildTagContext(messages: SessionMessage[]): string {
  const userMessages = messages.filter((m) => m.role === "user").slice(0, 5);
  if (userMessages.length === 0) return "(no user messages)";

  return userMessages
    .map((msg, idx) => `[${idx + 1}] ${truncateChars(msg.content.trim(), 500)}`)
    .join("\n---\n");
}

export async function loadAllTags(): Promise<Record<string, SessionTag>> {
  try {
    const raw = await invoke<string>("load_tags");
    const store = JSON.parse(raw) as SessionTagStore | Record<string, unknown>;
    const tags = (store as SessionTagStore).tags || {};

    for (const key of Object.keys(tagCache)) {
      delete tagCache[key];
    }

    for (const [key, value] of Object.entries(tags as Record<string, unknown>)) {
      if (!value || typeof value !== "object") continue;
      const normalized = normalizeTag(value as Record<string, unknown>, key);
      if (!normalized.sessionId) continue;
      tagCache[normalized.sessionId] = normalized;
    }

    return { ...tagCache };
  } catch (err) {
    console.error("Load tags error:", err);
    return {};
  }
}

export function getCachedTag(sessionId: string): SessionTag | undefined {
  return tagCache[sessionId];
}

export function getAllCategories(): string[] {
  const categories = new Set<string>();
  for (const tag of Object.values(tagCache)) {
    const category = tag.category.trim();
    if (category) categories.add(category);
  }
  return Array.from(categories).sort((a, b) => a.localeCompare(b, "zh-Hans-CN"));
}

export async function saveTag(sessionId: string, summary: string, category: string): Promise<SessionTag> {
  const raw = await invoke<string>("save_tag", { sessionId, summary, category });
  const parsed = JSON.parse(raw) as Record<string, unknown>;
  const normalized = normalizeTag(parsed, sessionId);
  tagCache[normalized.sessionId] = normalized;
  return normalized;
}

export async function tagSession(
  session: Session,
  messages: SessionMessage[],
  options?: {
    force?: boolean;
    provider?: "claude" | "gemini";
    model?: string;
    randomness?: number;
  }
): Promise<SessionTag> {
  const force = !!options?.force;
  if (!force && tagCache[session.id]) {
    return tagCache[session.id];
  }

  const raw = await invoke<string>("tag_session", {
    sessionId: session.id,
    title: sessionTitle(session),
    agent: session.agent,
    workspace: session.workspace,
    context: buildTagContext(messages),
    force,
    provider: options?.provider ?? null,
    model: options?.model ?? null,
    randomness: typeof options?.randomness === "number" ? options.randomness : null,
  });

  const parsed = JSON.parse(raw) as Record<string, unknown>;
  const normalized = normalizeTag(parsed, session.id);
  tagCache[normalized.sessionId] = normalized;
  return normalized;
}
