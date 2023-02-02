export {};

declare global {
    interface Window {
        ftd: any;
        enable_dark_mode(): void;
        enable_light_mode(): void;
        enable_system_mode(): void;
        [key: string]: any;
    }

    interface Export {
        init: object;
        data: object;
        handle_event(evt: Event, id: string, event: string, obj: Element): void;
        handle_function(evt: Event, id: string, event: string, obj: Element): any;
        set_string_for_all(variable: string, value: string): any;
        set_bool_for_all(variable: string, value: boolean): any;
        set_bool(id: string, variable: string, value: boolean): any;
        set_value(variable: string, value: any): any;
        set_value_by_id(id: string, variable: string, value: any): any;
        get_value(id: string, variable: string): any;
        is_empty(str: any): boolean;
        append(array: any[], value: any, args: any, data: any, id: string): any[]
    }

    interface String {
        format(...args: any[]): String;
        replace_format(...args: any[]): String;
    }
}
