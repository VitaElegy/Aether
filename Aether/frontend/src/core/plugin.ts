import type { Component } from 'vue';

export interface SelfSpacePlugin {
    id: string;
    label: string;
    icon: string; // RemixIcon class name (e.g., 'ri-book-line')
    component: Component;
    order?: number;
}
