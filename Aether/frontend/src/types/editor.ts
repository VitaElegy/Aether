export interface IEditorAdapter {
    load(content: any): Promise<void>;
    getValue(): any;
    export(format: 'markdown' | 'json'): Promise<string | Blob>;
    import(content: any, format: 'markdown' | 'json'): Promise<void>;
}
