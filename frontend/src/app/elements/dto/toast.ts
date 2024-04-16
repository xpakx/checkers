export interface Toast {
    message: String,
    id: String,
    time?: number,
    type: "info" | "error",
}