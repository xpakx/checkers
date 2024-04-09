export interface GameRequest {
    type: "AI" | "USER";
    opponent?: String;
    rules: "British";
    aiType?: "Random" | "None";
}