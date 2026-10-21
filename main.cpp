#include <iostream>
#include <optional>
#include <sstream>
#include <string>
#include <vector>

#include "standard_chess.hpp"
using namespace StandardChess;
void printBoard(const ChessState& s) {
    for (int r = 7; r >= 0; --r) {
        std::cout << r + 1 << " ";
        for (int f = 0; f < 8; ++f) {
            Piece p = s.board.at(f,r);
            std::cout << (p.type ? pieceChar(p) : '.') << ' ';
        }
        std::cout << '\n';
    }
    std::cout << "  a b c d e f g h\n";
}
void printStatus(ChessState& s) {
    GameResult result = outcome(s);
    switch (result.reason) {
        case None:
            std::cout << (s.turn == White ? "White" : "Black") << " to move";
            if (inCheck(s, s.turn)) std::cout << ", in check";
            if (claimThreefold(s)) std::cout << "; threefold draw claim available";
            if (claimFiftyMove(s)) std::cout << "; fifty-move draw claim available";
            std::cout << '\n'; break;
        case Checkmate: std::cout << "Checkmate: " << (result.winner == White ? "White" : "Black") << " wins\n"; break;
        case Stalemate: std::cout << "Draw: stalemate\n"; break;
        case ThreefoldClaim: std::cout << "Draw: threefold repetition claimed\n"; break;
        case FiftyMoveClaim: std::cout << "Draw: fifty-move rule claimed\n"; break;
        case Fivefold: std::cout << "Draw: fivefold repetition\n"; break;
        case SeventyFiveMove: std::cout << "Draw: seventy-five-move rule\n"; break;
        case DeadPosition: std::cout << "Draw: dead position\n"; break;
        case Agreement: std::cout << "Draw: agreement\n"; break;
        case Resignation: std::cout << (result.winner == White ? "White" : "Black") << " wins by resignation\n"; break;
    }
}
std::optional<Move> parseUci(std::string_view text) {
    if (text.size() != 4 && text.size() != 5) return std::nullopt;
    int from = parseSquare(text.substr(0,2)), to = parseSquare(text.substr(2,2));
    if (from < 0 || to < 0) return std::nullopt;
    int promotion = 0;
    if (text.size() == 5) {
        promotion = pieceType(text[4]);
        if (promotion != Queen && promotion != Rook && promotion != Bishop && promotion != Knight) return std::nullopt;
    }
    return Move{from, to, promotion};
}
int main() {
    ChessState state = Rules::initialPosition();
    std::string error;
    std::vector<UndoInfo> undoStack;
    std::cout << "Standard Chess rules engine. Type help for commands.\n";
    std::string line;
    while (std::cout << "> " && std::getline(std::cin, line)) {
        std::istringstream input(line);
        std::string command; input >> command;
        if (command.empty()) continue;
        if (command == "quit" || command == "exit") break;
        if (command == "help") {
            std::cout << "board | moves | move e2e4 | undo | fen | loadfen <FEN> | status | perft N | divide N | claim threefold|fifty | agree | resign | quit\n";
        } else if (command == "board") printBoard(state);
        else if (command == "fen") std::cout << fen(state) << '\n';
        else if (command == "status") printStatus(state);
        else if (command == "moves") {
            auto moves = legalMoves(state);
            for (Move m : moves) std::cout << uci(m) << ' ';
            std::cout << "\n" << moves.size() << " legal moves\n";
        } else if (command == "move") {
            std::string text, extra; input >> text;
            if (input >> extra) { std::cout << "Invalid move command\n"; continue; }
            auto move = parseUci(text);
            UndoInfo undo;
            if (!move || !makeAction(state, Action{Core::ActionKind::Move, *move}, undo)) std::cout << "Illegal move\n";
            else { undoStack.push_back(undo); printStatus(state); }
        } else if (command == "undo") {
            if (undoStack.empty()) std::cout << "Nothing to undo\n";
            else { unmakeAction(state, undoStack.back()); undoStack.pop_back(); std::cout << "Undone\n"; }
        } else if (command == "claim" || command == "agree" || command == "resign") {
            std::string reason, extra;
            input >> reason;
            int ruleCommand = command == "agree" ? AgreeDraw : Resign;
            if (command == "claim") {
                if (reason == "threefold") ruleCommand = ClaimThreefold;
                else if (reason == "fifty") ruleCommand = ClaimFiftyMove;
                else { std::cout << "Use claim threefold or claim fifty\n"; continue; }
            } else if (!reason.empty()) { std::cout << "Unexpected argument\n"; continue; }
            if (input >> extra) { std::cout << "Unexpected argument\n"; continue; }
            UndoInfo undo;
            if (!makeAction(state, Action{Core::ActionKind::RuleCommand, {}, ruleCommand}, undo)) std::cout << "Action unavailable\n";
            else { undoStack.push_back(undo); printStatus(state); }
        } else if (command == "loadfen") {
            std::string remainder; std::getline(input >> std::ws, remainder);
            ChessState loaded;
            if (!loadFen(loaded, remainder, error)) std::cout << "Invalid FEN: " << error << '\n';
            else { state = std::move(loaded); undoStack.clear(); std::cout << "FEN loaded\n"; }
        } else if (command == "perft" || command == "divide") {
            std::string count, extra; input >> count;
            int depth = -1;
            if (!parseInteger(count, 0, depth) || depth > 7 || (input >> extra)) {
                std::cout << "Depth must be an integer from 0 to 7\n"; continue;
            }
            if (command == "perft") std::cout << perft(state, depth) << " nodes\n";
            else {
                if (depth == 0) { std::cout << "1 nodes\n"; continue; }
                uint64_t total = 0;
                for (Move m : legalMoves(state)) {
                    UndoInfo undo = applyMove(state, m);
                    uint64_t nodes = perft(state, depth - 1);
                    unmakeAction(state, undo);
                    total += nodes;
                    std::cout << uci(m) << ": " << nodes << '\n';
                }
                std::cout << "Total: " << total << " nodes\n";
            }
        } else std::cout << "Unknown command. Type help.\n";
    }
}
