#include "standard_chess.hpp"
#include <iostream>
using namespace StandardChess;
#include <stdexcept>

void require(bool condition, const std::string& message) {
    if (!condition) throw std::runtime_error(message);
}
ChessState fromFen(std::string_view value) {
    ChessState s;
    std::string error;
    require(loadFen(s, std::string(value), error), "load FEN: " + error);
    return s;
}
bool sameState(const ChessState& a, const ChessState& b) {
    return a.board.width == b.board.width && a.board.height == b.board.height &&
           a.board.cells == b.board.cells && a.turn == b.turn && a.castling == b.castling &&
           a.epSquare == b.epSquare && a.halfmove == b.halfmove && a.fullmove == b.fullmove &&
           a.positionHistory == b.positionHistory && a.result == b.result;
}
bool hasMove(ChessState& s, std::string_view name) {
    for (Move m : legalMoves(s)) if (uci(m) == name) return true;
    return false;
}
void play(ChessState& s, std::string_view name) {
    Move m{parseSquare(name.substr(0, 2)), parseSquare(name.substr(2, 2)),
           name.size() == 5 ? pieceType(name[4]) : 0};
    UndoInfo u;
    require(makeAction(s, Action{Core::ActionKind::Move, m}, u), "play " + std::string(name));
}
void verifyUndo(ChessState s) {
    ChessState original = s;
    for (Move m : legalMoves(s)) {
        UndoInfo u;
        require(makeAction(s, Action{Core::ActionKind::Move, m}, u), "make legal move");
        unmakeAction(s, u);
        require(sameState(s, original), "make/unmake " + uci(m));
    }
}
int main() {
    try {
        ChessState start = fromFen(StartFen);
        require(fen(start) == StartFen, "starting FEN round trip");
        verifyUndo(start);
        for (auto [depth, expected] : {std::pair{1, 20ULL}, {2, 400ULL}, {3, 8902ULL}, {4, 197281ULL}, {5, 4865609ULL}})
            require(perft(start, depth) == expected, "starting perft depth " + std::to_string(depth));
        require(sameState(start, fromFen(StartFen)), "perft restores state");
        // Reference: Chess Programming Wiki, Perft Results, positions 2 and 3.
        ChessState kiwipete = fromFen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        require(perft(kiwipete, 4) == 4085603, "Kiwipete perft depth 4");
        ChessState position3 = fromFen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        require(perft(position3, 5) == 674624, "position 3 perft depth 5");

        ChessState castle = fromFen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        int kingsideRookId = castle.board.cells[static_cast<size_t>(sq(7,0))].id;
        require(hasMove(castle, "e1g1") && hasMove(castle, "e1c1"), "both castles available");
        verifyUndo(castle);
        play(castle, "e1g1");
        require(isPiece(castle.board.cells[static_cast<size_t>(sq(5,0))], Rook, White), "castling rook moved");
        require(castle.board.cells[static_cast<size_t>(sq(5,0))].id == kingsideRookId, "castling retains rook identity");
        require((castle.castling & (WK | WQ)) == 0, "castling rights cleared");
        ChessState noCastle = fromFen("4kr2/8/8/8/8/8/8/4K2R w K - 0 1");
        require(!hasMove(noCastle, "e1g1"), "castle through attacked transit square forbidden");
        ChessState inCheckCastle = fromFen("4k3/8/8/8/8/8/4r3/4K2R w K - 0 1");
        require(!hasMove(inCheckCastle, "e1g1"), "castle from check forbidden");

        ChessState ep = fromFen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        require(hasMove(ep, "e5d6"), "en passant available");
        verifyUndo(ep);
        play(ep, "e5d6");
        require(ep.board.cells[static_cast<size_t>(sq(3,4))].type == 0, "en passant removes pawn");
        ChessState epPin = fromFen("8/8/8/4KPpr/8/8/8/7k w - g6 0 1");
        require(!hasMove(epPin, "f5g6"), "en passant cannot expose king");
        require(effectiveEp(epPin) == -1, "illegal en passant omitted from repetition key");

        ChessState promotion = fromFen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1");
        int pawnId = promotion.board.cells[static_cast<size_t>(sq(0,6))].id;
        for (char c : std::string("qrbn")) require(hasMove(promotion, std::string("a7a8") + c), "promotion choice");
        require(!hasMove(promotion, "a7a8"), "promotion choice required");
        verifyUndo(promotion);
        play(promotion, "a7a8n");
        require(isPiece(promotion.board.cells[static_cast<size_t>(sq(0,7))], Knight, White), "promotion type");
        require(promotion.board.cells[static_cast<size_t>(sq(0,7))].id == pawnId, "promotion retains piece identity");

        ChessState pin = fromFen("k3r3/8/8/8/8/8/4R3/4K3 w - - 0 1");
        require(!hasMove(pin, "e2f2"), "pinned rook cannot leave file");
        ChessState check = fromFen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1");
        require(inCheck(check, White), "check detected");
        require(!hasMove(check, "e1d2"), "king cannot move into rook attack");
        require(hasMove(check, "e1e2"), "king may capture undefended rook");
        ChessState stalemate = fromFen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
        require(outcome(stalemate).reason == Stalemate, "stalemate");
        ChessState mate = fromFen("7k/6Q1/6K1/8/8/8/8/8 b - - 150 1");
        require(outcome(mate).reason == Checkmate, "mate precedes seventy-five move draw");
        ChessState dead = fromFen("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
        require(outcome(dead).reason == DeadPosition, "bare kings dead");

        ChessState repeat = fromFen(StartFen);
        const std::array<std::string_view, 4> cycle{"g1f3", "g8f6", "f3g1", "f6g8"};
        for (int i = 0; i < 2; ++i) for (auto m : cycle) play(repeat, m);
        require(claimThreefold(repeat) && outcome(repeat).kind == ResultKind::Ongoing, "threefold claim available");
        ChessState beforeClaim = repeat;
        UndoInfo claimUndo;
        require(makeAction(repeat, Action{Core::ActionKind::RuleCommand, {}, ClaimThreefold}, claimUndo), "threefold claim action");
        require(outcome(repeat).reason == ThreefoldClaim, "threefold claimed result");
        unmakeAction(repeat, claimUndo);
        require(sameState(repeat, beforeClaim), "claim undo restores state");
        for (int i = 0; i < 2; ++i) for (auto m : cycle) play(repeat, m);
        require(outcome(repeat).reason == Fivefold, "fivefold automatic draw");
        ChessState fifty = fromFen("4k3/8/8/8/8/8/8/4K2R w - - 99 1");
        play(fifty, "h1h2");
        require(claimFiftyMove(fifty), "fifty-move claim available");
        UndoInfo fiftyUndo;
        require(makeAction(fifty, Action{Core::ActionKind::RuleCommand, {}, ClaimFiftyMove}, fiftyUndo), "fifty-move claim action");
        require(outcome(fifty).reason == FiftyMoveClaim, "fifty-move claimed result");
        ChessState seventyFive = fromFen("4k3/8/8/8/8/8/8/4K2R w - - 149 1");
        play(seventyFive, "h1h2");
        require(outcome(seventyFive).reason == SeventyFiveMove, "seventy-five automatic draw");

        ChessState preserved = start;
        std::string error;
        require(!loadFen(preserved, "invalid", error) && sameState(preserved, start), "invalid FEN preserves state");
        ChessState epNoCapture = fromFen("4k3/8/8/3p4/8/8/8/4K3 w - d6 0 1");
        ChessState noEp = fromFen("4k3/8/8/3p4/8/8/8/4K3 w - - 0 1");
        require(positionKey(epNoCapture) == positionKey(noEp), "uncapturable EP same repetition position");
        ChessState resigned = fromFen(StartFen);
        UndoInfo resignUndo;
        require(makeAction(resigned, Action{Core::ActionKind::RuleCommand, {}, Resign}, resignUndo), "resign action");
        require(outcome(resigned) == GameResult{ResultKind::Win, Black, Resignation}, "resignation result");
        unmakeAction(resigned, resignUndo);
        require(sameState(resigned, start), "resignation undo");
        std::cout << "All tests passed\n";
    } catch (const std::exception& e) {
        std::cerr << "Test failed: " << e.what() << '\n';
        return 1;
    }
}
