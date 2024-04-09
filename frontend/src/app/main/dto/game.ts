export interface Game {
    id: number;
    invitation: "Issued" | "Accepted" | "Rejected";
    gameType: "User" | "AI";
    ruleset: "British";
    aiType: "Random" | "None";
    gameStatus: "NotFinished" | "Won" | "Lost" | "Drawn";
    currentState: String; // TODO
    lastMoveRow: number; // TODO
    lastMoveColumn: number; // TODO
    userStarts: boolean;
    userTurn: boolean;
    user: String;
    opponent: String; 
    user_id: number;
    opponent_id: number; 
}