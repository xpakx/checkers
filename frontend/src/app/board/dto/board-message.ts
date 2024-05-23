export interface BoardMessage {
    username1: String;
    username2: String;
    ai: boolean;

    currentState: ("WhitePawn" | "WhiteKing" | "RedPawn" | "RedKing" | "Empty")[][];
    currentPlayer: String;
    userTurn: boolean;
    status: "NotFinished" | "Won" | "Lost" | "Drawn";
}