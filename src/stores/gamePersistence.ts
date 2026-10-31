import { api } from "../services/api";
import type { GameDetail } from "../types/models";

export async function persistComment(
  data: GameDetail,
  index: number,
  comment: string,
): Promise<GameDetail> {
  await api.saveUserComment(data.game.id, index, comment);
  return {
    ...data,
    moves: data.moves.map((move) =>
      move.ply === index ? { ...move, userComment: comment } : move,
    ),
  };
}

export async function persistNotes(data: GameDetail, notes: string): Promise<GameDetail> {
  await api.updateNotes(data.game.id, notes);
  return { ...data, game: { ...data.game, notes } };
}
