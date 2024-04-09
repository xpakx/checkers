export interface GameRequest {
    type: "AI" | "User";
    opponent?: String;
    rules: "British";
    aiType?: "Random" | "None";
}