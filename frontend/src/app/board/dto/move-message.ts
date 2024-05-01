export interface MoveMessage {
    player: String;
    move: String;
    legal: boolean;

    details?: "Miss" | "Hit" | "Sunk";
    status: "NotFinished" | "Won" | "Lost" | "Drawn"; // TODO
}