// PLAT-03: Header Action Protocol — Type definitions

export interface HeaderAction {
  id: string;
  label: string;
  icon: string;           // Material icon name or custom SVG
  handler: () => void;
  badge?: HeaderBadge;
  disabled?: boolean;
  tooltip?: string;
  group?: string;         // Logical grouping (e.g., "export", "edit")
}

export interface HeaderBadge {
  type: 'progress' | 'count' | 'status' | 'context' | 'dot';
  value: number | string;
  color?: string;         // CSS color
  animate?: boolean;      // Pulse animation for active progress
  pulse?: boolean;        // Alias for animate (used in template)
  progress?: number;      // Progress percentage (0-100)
}

export interface HeaderActionGroup {
  moduleId: string;
  actions: HeaderAction[];
  priority: number;       // Higher = rendered first
}
