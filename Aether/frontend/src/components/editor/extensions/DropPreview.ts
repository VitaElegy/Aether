import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

// Defined outside to be accessible in decorations and commands
const dropPreviewKey = new PluginKey('dropPreview');

export interface DropPreviewOptions {
    class: string;
}

declare module '@tiptap/core' {
    interface Commands<ReturnType> {
        dropPreview: {
            setDropPreview: (pos: number, height: number, html: string) => ReturnType;
            clearDropPreview: () => ReturnType;
        }
    }
}

export const DropPreview = Extension.create<DropPreviewOptions>({
    name: 'dropPreview',

    addOptions() {
        return {
            class: 'drop-preview-widget',
        };
    },

    addCommands() {
        return {
            setDropPreview: (pos, height, html) => ({ tr, dispatch }) => {
                if (dispatch) {
                    tr.setMeta(dropPreviewKey, { pos, height, html });
                }
                return true;
            },
            clearDropPreview: () => ({ tr, dispatch }) => {
                if (dispatch) {
                    tr.setMeta(dropPreviewKey, { pos: null });
                }
                return true;
            },
        };
    },

    addProseMirrorPlugins() {
        const className = this.options.class;

        return [
            new Plugin({
                key: dropPreviewKey,
                state: {
                    init() {
                        return { pos: null, height: 0, html: '' };
                    },
                    apply(tr, value) {
                        const meta = tr.getMeta(dropPreviewKey);
                        if (meta) {
                            return meta;
                        }
                        if (value.pos !== null) {
                            return {
                                ...value,
                                pos: tr.mapping.map(value.pos)
                            };
                        }
                        return value;
                    },
                },
                props: {
                    decorations: (state) => {
                        try {
                            const pluginState = dropPreviewKey.getState(state);

                            if (!pluginState || pluginState.pos === null) {
                                return DecorationSet.empty;
                            }

                            const { pos, height, html } = pluginState;

                            // Validate pos is within bounds to prevent crash
                            if (pos < 0 || pos > state.doc.content.size) {
                                console.warn('DropPreview: pos out of bounds', pos);
                                return DecorationSet.empty;
                            }

                            const elem = document.createElement('div');
                            elem.className = className;

                            // Set styles for the ghost preview
                            elem.style.height = `${height}px`;
                            elem.style.width = '100%';
                            elem.style.position = 'relative';
                            elem.style.marginTop = '4px';
                            elem.style.marginBottom = '4px';
                            elem.style.opacity = '0.5';
                            elem.style.overflow = 'hidden';
                            elem.style.pointerEvents = 'none';
                            elem.style.backgroundColor = 'rgba(59, 130, 246, 0.05)';
                            elem.style.borderRadius = '4px';
                            elem.style.border = '1px dashed rgba(59, 130, 246, 0.4)';
                            elem.style.transition = 'height 0.2s ease, margin 0.2s ease';

                            const content = document.createElement('div');
                            content.innerHTML = html;

                            // Prevent any interactive elements in preview from capturing events
                            content.style.pointerEvents = 'none';

                            elem.appendChild(content);

                            return DecorationSet.create(state.doc, [
                                Decoration.widget(pos, elem, {
                                    key: 'drop-preview-widget',
                                    side: 0, // 0 = at position
                                }),
                            ]);
                        } catch (e) {
                            console.error('DropPreview decorations error:', e);
                            return DecorationSet.empty;
                        }
                    },
                },
            }),
        ];
    },
});
