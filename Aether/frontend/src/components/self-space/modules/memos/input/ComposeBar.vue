<script setup lang="ts">
import { ref, computed, nextTick } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { Icon } from 'tdesign-vue-next';

const store = useMemosStore();
const content = ref('');
const quickTags = ref<string[]>([]);
const showSlashMenu = ref(false);
const slashMenuPosition = ref(0);
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const pastedUrl = ref<string | null>(null);
const channelOverride = ref<string | null>(null);

const emit = defineEmits(['expand']);

// Slash commands for MEMO-02
const slashCommands = [
    { cmd: '/task', label: 'Create as Task', icon: 'task', action: () => setQuickStatus('Todo') },
    { cmd: '/done', label: 'Mark Done', icon: 'check-circle', action: () => setQuickStatus('Done') },
    { cmd: '/pin', label: 'Pin this memo', icon: 'pin', action: () => setQuickPin(true) },
    { cmd: '/p0', label: 'Priority: Urgent', icon: 'flag', action: () => setQuickPriority('P0') },
    { cmd: '/p1', label: 'Priority: High', icon: 'flag', action: () => setQuickPriority('P1') },
    { cmd: '/due', label: 'Set due date', icon: 'calendar', action: () => openDueDatePicker() },
    { cmd: '/channel', label: 'Set channel', icon: 'folder', action: () => showChannelPicker() },
    { cmd: '/remind', label: 'Set reminder', icon: 'time', action: () => openReminderPicker() },
];

const quickStatus = ref<string | null>(null);
const quickPinned = ref(false);
const quickPriority = ref<string | null>(null);
const quickDueAt = ref<string | null>(null);
const quickReminder = ref<string | null>(null);
const showChannelMenu = ref(false);

function setQuickStatus(s: string) { quickStatus.value = s; closeSlashMenu(); }
function setQuickPin(v: boolean) { quickPinned.value = v; closeSlashMenu(); }
function setQuickPriority(p: string) { quickPriority.value = p; closeSlashMenu(); }
function openDueDatePicker() { closeSlashMenu(); /* Simple prompt for now */ const d = prompt('Due date (YYYY-MM-DD):'); if (d) quickDueAt.value = new Date(d).toISOString(); }
function openReminderPicker() { closeSlashMenu(); const d = prompt('Reminder (YYYY-MM-DD HH:mm):'); if (d) quickReminder.value = new Date(d).toISOString(); }
function showChannelPicker() { closeSlashMenu(); showChannelMenu.value = !showChannelMenu.value; }

function closeSlashMenu() {
    showSlashMenu.value = false;
    // Remove slash command text from content
    content.value = content.value.replace(/\/\w*$/, '').trim();
}

// Filtered slash commands
const filteredCommands = computed(() => {
    const match = content.value.match(/\/(\w*)$/);
    if (!match) return [];
    const query = match[1].toLowerCase();
    return slashCommands.filter(c => c.cmd.toLowerCase().includes('/' + query));
});

function handleInput() {
    // Check for slash command trigger
    if (content.value.match(/\/\w*$/)) {
        showSlashMenu.value = true;
    } else {
        showSlashMenu.value = false;
    }

    // Auto-detect tags (#tag)
    const tagMatches = content.value.match(/#(\w+)/g);
    if (tagMatches) {
        quickTags.value = tagMatches.map(t => t.slice(1));
    }

    // Auto-detect pasted URLs
    const urlMatch = content.value.match(/(https?:\/\/[^\s]+)/);
    if (urlMatch && !pastedUrl.value) {
        pastedUrl.value = urlMatch[1];
    }

    // Auto-resize textarea
    autoResize();
}

function autoResize() {
    if (textareaRef.value) {
        textareaRef.value.style.height = 'auto';
        textareaRef.value.style.height = Math.min(textareaRef.value.scrollHeight, 160) + 'px';
    }
}

function executeSlashCommand(cmd: typeof slashCommands[0]) {
    cmd.action();
}

function expand() {
    store.openEditor();
}

async function handleSubmit() {
    if (!content.value.trim()) return;

    // Extract inline tags from content
    const inlineTags = content.value.match(/#(\w+)/g)?.map(t => t.slice(1)) || [];

    // Combine with filter context tags
    const tags = store.filterTags.length > 0
        ? [...new Set([...store.filterTags, ...inlineTags])]
        : inlineTags;

    // Clean content: remove inline tag syntax for cleaner storage
    let cleanContent = content.value;

    const title = cleanContent.substring(0, 50) + (cleanContent.length > 50 ? '...' : '');

    try {
        await store.createMemo({
            title,
            content: cleanContent,
            tags,
            visibility: 'Private',
            status: quickStatus.value || undefined,
            priority: quickPriority.value || undefined,
            is_pinned: quickPinned.value || undefined,
            due_at: quickDueAt.value || undefined,
            reminder_at: quickReminder.value || undefined,
            channel: channelOverride.value || (store.filterChannel || undefined),
        });
        // Reset
        content.value = '';
        quickTags.value = [];
        quickStatus.value = null;
        quickPriority.value = null;
        quickPinned.value = false;
        quickDueAt.value = null;
        quickReminder.value = null;
        pastedUrl.value = null;
        channelOverride.value = null;
        if (textareaRef.value) textareaRef.value.style.height = 'auto';
    } catch (e) {
        console.error('Failed to create memo', e);
    }
}

function selectChannel(ch: string) {
    channelOverride.value = ch;
    showChannelMenu.value = false;
}

// Active context badges
const activeBadges = computed(() => {
    const badges: { label: string; color: string; clear: () => void }[] = [];
    if (quickStatus.value) badges.push({ label: quickStatus.value, color: 'bg-green-50 text-green-600', clear: () => quickStatus.value = null });
    if (quickPriority.value) badges.push({ label: quickPriority.value, color: 'bg-red-50 text-red-600', clear: () => quickPriority.value = null });
    if (quickPinned.value) badges.push({ label: 'Pinned', color: 'bg-amber-50 text-amber-600', clear: () => quickPinned.value = false });
    if (quickDueAt.value) badges.push({ label: 'Due set', color: 'bg-blue-50 text-blue-600', clear: () => quickDueAt.value = null });
    if (channelOverride.value) badges.push({ label: '#' + channelOverride.value, color: 'bg-indigo-50 text-indigo-600', clear: () => channelOverride.value = null });
    return badges;
});
</script>

<template>
    <div class="border-t border-gray-100 bg-white p-4 shrink-0 shadow-sm z-10">
        <div class="max-w-4xl mx-auto">
            <!-- Active Context Badges -->
            <div v-if="activeBadges.length > 0" class="flex flex-wrap gap-1.5 mb-2">
                <span
                    v-for="(badge, i) in activeBadges"
                    :key="i"
                    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wider"
                    :class="badge.color"
                >
                    {{ badge.label }}
                    <button @click="badge.clear()" class="hover:opacity-70">×</button>
                </span>
            </div>

            <!-- URL Card Preview -->
            <div v-if="pastedUrl" class="mb-2 bg-gray-50 rounded-lg px-3 py-2 flex items-center gap-2 text-xs text-gray-500 border border-gray-200">
                <Icon name="link" size="14px" class="text-gray-400" />
                <span class="truncate flex-1">{{ pastedUrl }}</span>
                <button @click="pastedUrl = null" class="text-gray-400 hover:text-gray-600">
                    <Icon name="close" size="12px" />
                </button>
            </div>

            <!-- Main Input Area -->
            <div class="relative items-end gap-2 flex bg-gray-50 border border-gray-200 rounded-xl p-2 focus-within:ring-2 focus-within:ring-blue-50 transition-all shadow-sm hover:shadow-md">

                <!-- Expand -->
                <button
                    @click="expand"
                    class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors shrink-0"
                    title="Open Full Editor"
                >
                    <Icon name="fullscreen" size="20px" />
                </button>

                <!-- Input -->
                <textarea
                    ref="textareaRef"
                    v-model="content"
                    @input="handleInput"
                    class="flex-1 bg-transparent border-none focus:ring-0 resize-none py-2 text-sm text-gray-700 placeholder-gray-400 min-h-[40px] max-h-40 focus:outline-none custom-scrollbar"
                    placeholder="What's on your mind? Type / for commands, # for tags"
                    rows="1"
                    @keydown.enter.exact.prevent="handleSubmit"
                ></textarea>

                <!-- Actions -->
                <div class="flex items-center gap-1 shrink-0">
                    <!-- Channel selector -->
                    <div class="relative">
                        <button
                            @click="showChannelMenu = !showChannelMenu"
                            class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors"
                            :class="{ 'text-indigo-500': channelOverride }"
                            title="Set Channel"
                        >
                            <Icon name="folder" size="20px" />
                        </button>
                        <div v-if="showChannelMenu" class="absolute bottom-full right-0 mb-2 w-40 bg-white shadow-lg rounded-lg border border-gray-200 py-1 z-20">
                            <button
                                v-for="ch in store.channels"
                                :key="ch"
                                @click="selectChannel(ch)"
                                class="w-full text-left px-3 py-1.5 text-xs text-gray-600 hover:bg-gray-50"
                            >
                                {{ ch }}
                            </button>
                            <div v-if="store.channels.length === 0" class="px-3 py-2 text-xs text-gray-400">No channels yet</div>
                        </div>
                    </div>

                    <button class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors" title="Attach">
                        <Icon name="attach" size="20px" />
                    </button>

                    <button
                        @click="handleSubmit"
                        :disabled="!content.trim()"
                        class="p-2 rounded-lg transition-all flex items-center justify-center w-9 h-9"
                        :class="content.trim() ? 'bg-blue-600 text-white hover:bg-blue-700 shadow-sm' : 'bg-gray-100 text-gray-300 cursor-not-allowed'"
                    >
                        <Icon name="send" size="18px" />
                    </button>
                </div>

                <!-- Slash Command Menu -->
                <Transition name="fade">
                    <div
                        v-if="showSlashMenu && filteredCommands.length > 0"
                        class="absolute bottom-full left-12 mb-2 w-56 bg-white shadow-xl rounded-xl border border-gray-100 py-1 z-30"
                    >
                        <button
                            v-for="cmd in filteredCommands"
                            :key="cmd.cmd"
                            @click="executeSlashCommand(cmd)"
                            class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                        >
                            <Icon :name="cmd.icon" size="16px" class="text-gray-400" />
                            <div>
                                <div class="font-medium">{{ cmd.label }}</div>
                                <div class="text-[10px] text-gray-400">{{ cmd.cmd }}</div>
                            </div>
                        </button>
                    </div>
                </Transition>
            </div>

            <div class="mt-2 flex justify-between text-[10px] text-gray-400 px-3 uppercase tracking-wider font-medium">
                <span>Context: {{ store.filterChannel ? '📁 ' + store.filterChannel : store.filterTags.length ? '#' + store.filterTags[0] : 'All' }}</span>
                <span>Enter to send · / for commands · # for tags</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
    transition: opacity 0.15s ease, transform 0.15s ease;
}
.fade-enter-from, .fade-leave-to {
    opacity: 0;
    transform: translateY(4px);
}
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e7eb; border-radius: 4px; }
</style>
