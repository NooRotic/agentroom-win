/**
 * useAgentEvents — Replaces useExtensionMessages.ts
 * Listens to Tauri 'agent-state-changed' events and drives OfficeState.
 */
import { useState, useEffect, useCallback, useRef } from 'react'
import type { OfficeState } from '../office/engine/officeState.js'
import type { ToolActivity } from '../office/types.js'
import { extractToolName } from '../office/toolUtils.js'
import { listenAgentEvents, type AgentEventPayload } from '../bridge.js'
import { playDoneSound } from '../office/notificationSound.js'

export interface SubagentCharacter {
  id: number
  parentAgentId: number
  parentToolId: string
  label: string
}

export interface AgentEventState {
  agents: number[]
  agentTools: Record<number, ToolActivity[]>
  agentStatuses: Record<number, string>
  subagentCharacters: SubagentCharacter[]
}

// Map string agent_id (session UUID) to numeric ID for the game engine
let nextAgentNumericId = 1
const agentIdMap = new Map<string, number>()

function getOrCreateNumericId(agentId: string): number {
  let numId = agentIdMap.get(agentId)
  if (numId === undefined) {
    numId = nextAgentNumericId++
    agentIdMap.set(agentId, numId)
  }
  return numId
}

export function useAgentEvents(
  getOfficeState: () => OfficeState,
): AgentEventState {
  const [agents, setAgents] = useState<number[]>([])
  const [agentTools, setAgentTools] = useState<Record<number, ToolActivity[]>>({})
  const [agentStatuses, setAgentStatuses] = useState<Record<number, string>>({})
  const [subagentCharacters, setSubagentCharacters] = useState<SubagentCharacter[]>([])

  // Track active tool counts per agent for tool_done logic
  const activeToolCountRef = useRef<Map<number, Set<string>>>(new Map())

  const ensureAgent = useCallback((numId: number) => {
    const os = getOfficeState()
    if (!os.characters.has(numId)) {
      os.addAgent(numId)
      setAgents((prev) => prev.includes(numId) ? prev : [...prev, numId])
    }
  }, [getOfficeState])

  useEffect(() => {
    const unlistenPromise = listenAgentEvents((event: AgentEventPayload) => {
      const os = getOfficeState()
      const numId = getOrCreateNumericId(event.agent_id)

      switch (event.status) {
        case 'tool_start': {
          ensureAgent(numId)
          const toolId = event.tool_id || ''
          const status = event.tool_status || `Using ${event.tool_name || 'tool'}`

          // Track active tools
          let toolSet = activeToolCountRef.current.get(numId)
          if (!toolSet) {
            toolSet = new Set()
            activeToolCountRef.current.set(numId, toolSet)
          }
          toolSet.add(toolId)

          setAgentTools((prev) => {
            const list = prev[numId] || []
            if (list.some((t) => t.toolId === toolId)) return prev
            return { ...prev, [numId]: [...list, { toolId, status, done: false }] }
          })

          const toolName = extractToolName(status)
          os.setAgentTool(numId, toolName)
          os.setAgentActive(numId, true)
          os.clearPermissionBubble(numId)

          // Sub-agent creation for Task tool
          if (event.is_subagent && event.parent_tool_id) {
            const label = status.startsWith('Subtask:') ? status.slice('Subtask:'.length).trim() : status
            const subId = os.addSubagent(numId, event.parent_tool_id)
            setSubagentCharacters((prev) => {
              if (prev.some((s) => s.id === subId)) return prev
              return [...prev, { id: subId, parentAgentId: numId, parentToolId: event.parent_tool_id!, label }]
            })
          }

          setAgentStatuses((prev) => {
            if (!(numId in prev)) return prev
            const next = { ...prev }
            delete next[numId]
            return next
          })
          break
        }

        case 'tool_done': {
          const toolId = event.tool_id || ''

          // Remove from active tools
          const toolSet = activeToolCountRef.current.get(numId)
          if (toolSet) {
            toolSet.delete(toolId)
            // If no more active tools, clear tool animation
            if (toolSet.size === 0) {
              os.setAgentTool(numId, null)
            }
          }

          setAgentTools((prev) => {
            const list = prev[numId]
            if (!list) return prev
            return {
              ...prev,
              [numId]: list.map((t) => (t.toolId === toolId ? { ...t, done: true } : t)),
            }
          })

          // Handle sub-agent tool done
          if (event.is_subagent && event.parent_tool_id) {
            const subId = os.getSubagentId(numId, event.parent_tool_id)
            if (subId !== null) {
              os.setAgentTool(subId, null)
            }
          }
          break
        }

        case 'turn_end': {
          ensureAgent(numId)

          // Clear all tools
          activeToolCountRef.current.delete(numId)
          setAgentTools((prev) => {
            if (!(numId in prev)) return prev
            const next = { ...prev }
            delete next[numId]
            return next
          })

          // Remove sub-agents
          os.removeAllSubagents(numId)
          setSubagentCharacters((prev) => prev.filter((s) => s.parentAgentId !== numId))

          os.setAgentTool(numId, null)
          os.setAgentActive(numId, false)
          os.clearPermissionBubble(numId)

          setAgentStatuses((prev) => ({ ...prev, [numId]: 'waiting' }))
          os.showWaitingBubble(numId)
          playDoneSound()
          break
        }

        case 'waiting': {
          ensureAgent(numId)
          os.setAgentActive(numId, false)
          os.showWaitingBubble(numId)
          setAgentStatuses((prev) => ({ ...prev, [numId]: 'waiting' }))
          playDoneSound()
          break
        }

        case 'active': {
          ensureAgent(numId)
          os.setAgentActive(numId, true)
          setAgentStatuses((prev) => {
            if (!(numId in prev)) return prev
            const next = { ...prev }
            delete next[numId]
            return next
          })
          break
        }

        case 'permission': {
          ensureAgent(numId)
          os.showPermissionBubble(numId)
          setAgentTools((prev) => {
            const list = prev[numId]
            if (!list) return prev
            return {
              ...prev,
              [numId]: list.map((t) => (t.done ? t : { ...t, permissionWait: true })),
            }
          })
          break
        }

        case 'permission_clear': {
          os.clearPermissionBubble(numId)
          // Also clear sub-agent permission bubbles
          for (const [subId, meta] of os.subagentMeta) {
            if (meta.parentAgentId === numId) {
              os.clearPermissionBubble(subId)
            }
          }
          setAgentTools((prev) => {
            const list = prev[numId]
            if (!list) return prev
            const hasPermission = list.some((t) => t.permissionWait)
            if (!hasPermission) return prev
            return {
              ...prev,
              [numId]: list.map((t) => (t.permissionWait ? { ...t, permissionWait: false } : t)),
            }
          })
          break
        }

        case 'text_idle': {
          ensureAgent(numId)
          os.setAgentActive(numId, false)
          os.showWaitingBubble(numId)
          setAgentStatuses((prev) => ({ ...prev, [numId]: 'waiting' }))
          playDoneSound()
          break
        }
      }
    })

    return () => {
      unlistenPromise.then((fn) => fn())
    }
  }, [getOfficeState, ensureAgent])

  return { agents, agentTools, agentStatuses, subagentCharacters }
}
