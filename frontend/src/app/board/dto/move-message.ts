import { MoveDetails } from "./move-details";

export interface MoveMessage {
    player: String;
    move: String;
    legal: boolean;

    details?: MoveDetails;
    status: "NotFinished" | "Won" | "Lost" | "Drawn"; // TODO
}