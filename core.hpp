#pragma once
#include <concepts>
#include <cstddef>
#include <limits>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace Core {
struct Position { int file = 0, rank = 0; bool operator==(const Position&) const = default; };
using PieceType = int;
using PlayerId = int;
struct Piece { PieceType type = 0; PlayerId owner = -1; int id = 0; bool operator==(const Piece&) const = default; };
struct Board {
    int width, height;
    std::vector<Piece> cells;
    // Type 0 is empty. Modules supply their own identifiers and definitions.
    std::vector<std::string> pieceNames;
    std::vector<std::string> colorNames;
    // Optional direction per color: +1 or -1 for relative Chessembly vectors.
    std::vector<int> colorDirections;
    // Optional playable-cell mask, row-major; empty means a full rectangle.
    std::vector<unsigned char> boundary;
    Board(int w, int h) : width(w), height(h) {
        if (w <= 0 || h <= 0 || static_cast<long long>(w) * h > std::numeric_limits<int>::max())
            throw std::invalid_argument("invalid board dimensions");
        cells.resize(static_cast<size_t>(w) * static_cast<size_t>(h));
    }
    bool inside(int f, int r) const {
        return f >= 0 && f < width && r >= 0 && r < height &&
            (boundary.empty() || (boundary.size() == cells.size() && boundary[static_cast<size_t>(index(f,r))] != 0));
    }
    int index(int f, int r) const { return r * width + f; }
    Piece at(int f, int r) const {
        if (!inside(f, r)) throw std::out_of_range("board position");
        return cells[static_cast<size_t>(index(f, r))];
    }
    Piece& at(int f, int r) {
        if (!inside(f, r)) throw std::out_of_range("board position");
        return cells[static_cast<size_t>(index(f, r))];
    }
    int direction(Piece p) const {
        if (p.owner < 0 || static_cast<size_t>(p.owner) >= colorNames.size()) throw std::invalid_argument("undefined piece color");
        if (colorDirections.empty()) return 1;
        if (colorDirections.size() != colorNames.size() || (colorDirections[static_cast<size_t>(p.owner)] != 1 && colorDirections[static_cast<size_t>(p.owner)] != -1))
            throw std::invalid_argument("invalid color directions");
        return colorDirections[static_cast<size_t>(p.owner)];
    }
    const std::string& pieceName(Piece p) const {
        if (p.type <= 0 || static_cast<size_t>(p.type) >= pieceNames.size()) throw std::invalid_argument("undefined piece type");
        return pieceNames[static_cast<size_t>(p.type)];
    }
    const std::string& colorName(Piece p) const {
        if (p.owner < 0 || static_cast<size_t>(p.owner) >= colorNames.size()) throw std::invalid_argument("undefined piece color");
        return colorNames[static_cast<size_t>(p.owner)];
    }
};
enum class ActionKind { Move, RuleCommand };
struct Move { int from = -1, to = -1, promotion = 0; bool operator==(const Move&) const = default; };
struct Action { ActionKind kind = ActionKind::Move; Move move; int command = 0; bool operator==(const Action&) const = default; };
enum class ResultKind { Ongoing, Win, Draw };
struct GameResult { ResultKind kind = ResultKind::Ongoing; int winner = -1; int reason = 0; bool operator==(const GameResult&) const = default; };
struct GameState {
    Board board;
    int turn = 0;
    GameResult result;
    GameState(int width, int height) : board(width, height) {}
};
// Piece definitions may have any module-owned type. Only the four module entry
// points and the board/action interchange types are standardized.
template<class M>
concept RuleModule = requires(typename M::State& state) {
    { M::pieceDefinitions() };
    { M::boardBoundary() } -> std::same_as<Board>;
    { M::initialPosition() } -> std::same_as<typename M::State>;
    { M::victoryCondition(state) } -> std::same_as<GameResult>;
};
} // namespace Core
