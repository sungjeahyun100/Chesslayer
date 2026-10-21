#pragma once
#include "core.hpp"
#include "chessembly.hpp"
#include <algorithm>
#include <array>
#include <charconv>
#include <cctype>
#include <cstdint>
#include <cstdlib>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace StandardChess {
using Core::Action;
using Core::Board;
using Core::Move;
using Core::Piece;
using Core::GameResult;
using Core::ResultKind;
constexpr int White = 0, Black = 1;
constexpr int Pawn = 1, Knight = 2, Bishop = 3, Rook = 4, Queen = 5, King = 6;
constexpr unsigned WK = 1, WQ = 2, BK = 4, BQ = 8;
enum ChessCommand { ClaimThreefold = 1, ClaimFiftyMove, AgreeDraw, Resign };
enum ResultReason { None = 0, Checkmate, Stalemate, ThreefoldClaim, FiftyMoveClaim,
                    Fivefold, SeventyFiveMove, DeadPosition, Agreement, Resignation };
constexpr std::string_view StartFen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
inline int sq(int f, int r) { return r * 8 + f; }
inline int file(int s) { return s % 8; }
inline int rank(int s) { return s / 8; }
inline int opponent(int side) { return side ^ 1; }
inline bool inside(int f, int r) { return f >= 0 && f < 8 && r >= 0 && r < 8; }
inline bool isPiece(Piece piece, int type, int owner) { return piece.type == type && piece.owner == owner; }

// Only this game definition owns castling, en passant, chess clocks, and repetition.
struct ChessState : Core::GameState {
    ChessState() : GameState(8, 8) {
        board.pieceNames = {"", "pawn", "knight", "bishop", "rook", "queen", "king"};
        board.colorNames = {"white", "black"};
        board.colorDirections = {1, -1};
    }
    unsigned castling = 0;
    int epSquare = -1;
    int halfmove = 0, fullmove = 1;
    std::vector<std::string> positionHistory;
};

// An undo stores at most four changed squares, plus prior scalar state. No board copy.
struct UndoInfo {
    std::array<int, 4> squares{};
    std::array<Piece, 4> before{};
    int count = 0;
    int turn = White, epSquare = -1, halfmove = 0, fullmove = 1;
    unsigned castling = 0;
    size_t historySize = 0;
    GameResult result;
};

inline char pieceChar(Piece p) {
    constexpr std::array<char, 7> chars{' ', 'p', 'n', 'b', 'r', 'q', 'k'};
    char c = chars[static_cast<size_t>(p.type)];
    return p.owner == White ? static_cast<char>(std::toupper(static_cast<unsigned char>(c))) : c;
}
inline int pieceType(char c) {
    switch (std::tolower(static_cast<unsigned char>(c))) {
        case 'p': return Pawn; case 'n': return Knight; case 'b': return Bishop;
        case 'r': return Rook; case 'q': return Queen; case 'k': return King;
        default: return 0;
    }
}
inline std::string squareName(int s) { return {static_cast<char>('a' + file(s)), static_cast<char>('1' + rank(s))}; }
inline int parseSquare(std::string_view text) {
    if (text.size() != 2 || text[0] < 'a' || text[0] > 'h' || text[1] < '1' || text[1] > '8') return -1;
    return sq(text[0] - 'a', text[1] - '1');
}
inline std::string uci(Move m) {
    std::string out = squareName(m.from) + squareName(m.to);
    if (m.promotion) out += pieceChar({m.promotion, Black});
    return out;
}

// Attack tests describe control of a square. Pawn attacks differ from pawn moves,
// and a pinned attacker still attacks its target for king safety and castling.
inline bool attacked(const ChessState& s, int target, int by) {
    const auto& b = s.board.cells;
    const int f = file(target), r = rank(target);
    const int pawnRank = r - (by == White ? 1 : -1);
    for (int df : {-1, 1}) {
        if (inside(f + df, pawnRank) && isPiece(b[static_cast<size_t>(sq(f + df, pawnRank))], Pawn, by)) return true;
    }
    constexpr std::array<std::pair<int,int>, 8> knights{{{1,2},{2,1},{2,-1},{1,-2},{-1,-2},{-2,-1},{-2,1},{-1,2}}};
    for (auto [df, dr] : knights)
        if (inside(f + df, r + dr) && isPiece(b[static_cast<size_t>(sq(f + df, r + dr))], Knight, by)) return true;
    constexpr std::array<std::pair<int,int>, 8> dirs{{{1,0},{-1,0},{0,1},{0,-1},{1,1},{1,-1},{-1,1},{-1,-1}}};
    for (int i = 0; i < 8; ++i) {
        auto [df, dr] = dirs[static_cast<size_t>(i)];
        for (int step = 1; inside(f + df * step, r + dr * step); ++step) {
            Piece p = b[static_cast<size_t>(sq(f + df * step, r + dr * step))];
            if (!p.type) continue;
            if (p.owner == by && (p.type == Queen || (i < 4 && p.type == Rook) ||
                                  (i >= 4 && p.type == Bishop) || (step == 1 && p.type == King))) return true;
            break;
        }
    }
    return false;
}
inline int kingSquare(const ChessState& s, int side) {
    for (int i = 0; i < 64; ++i) if (isPiece(s.board.cells[static_cast<size_t>(i)], King, side)) return i;
    return -1;
}
inline bool inCheck(const ChessState& s, int side) {
    int k = kingSquare(s, side);
    return k < 0 || attacked(s, k, opponent(side));
}

inline void addPawnMove(std::vector<Move>& out, int from, int to, int side) {
    if (rank(to) == (side == White ? 7 : 0)) {
        for (int type : {Queen, Rook, Bishop, Knight}) out.push_back({from, to, type});
    } else out.push_back({from, to, 0});
}
// The full Chesssembly source for ordinary pieces belongs to this module.
// Pawn movement and its state-dependent rules are native to StandardChess.
struct PieceDefinition { std::string_view name, chessembly; bool nativeMovement; };
inline constexpr std::array<PieceDefinition,6> ChessPieces{{
    {"pawn", "", true},
    {"knight", "take-move(1,2);take-move(2,1);take-move(2,-1);take-move(1,-2);"
               "take-move(-1,-2);take-move(-2,-1);take-move(-2,1);take-move(-1,2);", false},
    {"bishop", "take-move(1,1) repeat(1);take-move(1,-1) repeat(1);"
               "take-move(-1,1) repeat(1);take-move(-1,-1) repeat(1);", false},
    {"rook", "take-move(1,0) repeat(1);take-move(-1,0) repeat(1);"
             "take-move(0,1) repeat(1);take-move(0,-1) repeat(1);", false},
    {"queen", "take-move(1,0) repeat(1);take-move(-1,0) repeat(1);"
              "take-move(0,1) repeat(1);take-move(0,-1) repeat(1);"
              "take-move(1,1) repeat(1);take-move(1,-1) repeat(1);"
              "take-move(-1,1) repeat(1);take-move(-1,-1) repeat(1);", false},
    {"king", "take-move(1,0);take-move(-1,0);take-move(0,1);take-move(0,-1);"
             "take-move(1,1);take-move(1,-1);take-move(-1,1);take-move(-1,-1);", false}
}};
inline const Chessembly::Program& knightProgram() { static const Chessembly::Program p(std::string(ChessPieces[1].chessembly)); return p; }
inline const Chessembly::Program& bishopProgram() { static const Chessembly::Program p(std::string(ChessPieces[2].chessembly)); return p; }
inline const Chessembly::Program& rookProgram() { static const Chessembly::Program p(std::string(ChessPieces[3].chessembly)); return p; }
inline const Chessembly::Program& queenProgram() { static const Chessembly::Program p(std::string(ChessPieces[4].chessembly)); return p; }
inline const Chessembly::Program& kingProgram() { static const Chessembly::Program p(std::string(ChessPieces[5].chessembly)); return p; }
// Pseudo-legal moves obey piece movement and occupancy. King safety is a later filter.
inline std::vector<Move> pseudoMoves(const ChessState& s) {
    std::vector<Move> out;
    out.reserve(64);
    const auto& b = s.board.cells;
    const int side = s.turn;
    for (int from = 0; from < 64; ++from) {
        Piece p = b[static_cast<size_t>(from)];
        if (!p.type || p.owner != side) continue;
        int f = file(from), r = rank(from);
        if (p.type == Pawn) {
            int dr = side == White ? 1 : -1;
            int next = sq(f, r + dr);
            if (inside(f, r + dr) && !b[static_cast<size_t>(next)].type) {
                addPawnMove(out, from, next, side);
                int two = sq(f, r + 2 * dr);
                if (r == (side == White ? 1 : 6) && !b[static_cast<size_t>(two)].type)
                    out.push_back({from, two, 0});
            }
            for (int df : {-1, 1}) {
                if (!inside(f + df, r + dr)) continue;
                int to = sq(f + df, r + dr);
                Piece victim = b[static_cast<size_t>(to)];
                if ((victim.type && victim.owner != side && victim.type != King) ||
                    (to == s.epSquare && !victim.type && isPiece(b[static_cast<size_t>(sq(f + df, r))], Pawn, opponent(side))))
                    addPawnMove(out, from, to, side);
            }
            continue;
        }
        // Ordinary piece movement comes from the shared Chessembly interpreter.
        // Standard Chess owns king safety, pawn rules, and castling below/above.
        const Chessembly::Program* program = nullptr;
        if (p.type == Knight) program = &knightProgram();
        else if (p.type == Bishop) program = &bishopProgram();
        else if (p.type == Rook) program = &rookProgram();
        else if (p.type == Queen) program = &queenProgram();
        else if (p.type == King) program = &kingProgram();
        if (!program) continue;
        for (const auto& action : program->execute(s.board, {f, r})) {
            if (!action.continuation.empty()) continue;
            const auto& step = action.first;
            int to = sq(step.moveTo.first, step.moveTo.second);
            if (b[static_cast<size_t>(to)].type == King) continue;
            out.push_back({from, to, 0});
        }
    }
    // Castling additionally requires an unattacked starting and transit square.
    int home = side == White ? 0 : 7;
    int k = sq(4, home), enemy = opponent(side);
    unsigned ks = side == White ? WK : BK, qs = side == White ? WQ : BQ;
    if (isPiece(b[static_cast<size_t>(k)], King, side) && !attacked(s, k, enemy)) {
        if ((s.castling & ks) && isPiece(b[static_cast<size_t>(sq(7, home))], Rook, side) &&
            !b[static_cast<size_t>(sq(5, home))].type && !b[static_cast<size_t>(sq(6, home))].type &&
            !attacked(s, sq(5, home), enemy)) out.push_back({k, sq(6, home), 0});
        if ((s.castling & qs) && isPiece(b[static_cast<size_t>(sq(0, home))], Rook, side) &&
            !b[static_cast<size_t>(sq(1, home))].type && !b[static_cast<size_t>(sq(2, home))].type &&
            !b[static_cast<size_t>(sq(3, home))].type && !attacked(s, sq(3, home), enemy))
            out.push_back({k, sq(2, home), 0});
    }
    return out;
}

inline void remember(ChessState& s, UndoInfo& u, int square) {
    for (int i = 0; i < u.count; ++i) if (u.squares[static_cast<size_t>(i)] == square) return;
    u.squares[static_cast<size_t>(u.count)] = square;
    u.before[static_cast<size_t>(u.count++)] = s.board.cells[static_cast<size_t>(square)];
}
// Private transition: callers must pass a generated move. Public makeAction validates first.
inline UndoInfo applyMove(ChessState& s, Move m) {
    UndoInfo u;
    u.turn = s.turn; u.castling = s.castling; u.epSquare = s.epSquare;
    u.halfmove = s.halfmove; u.fullmove = s.fullmove;
    u.historySize = s.positionHistory.size(); u.result = s.result;
    auto& b = s.board.cells;
    Piece p = b[static_cast<size_t>(m.from)], target = b[static_cast<size_t>(m.to)];
    remember(s, u, m.from); remember(s, u, m.to);
    bool epCapture = p.type == Pawn && m.to == s.epSquare && !target.type && file(m.from) != file(m.to);
    if (epCapture) {
        int captured = sq(file(m.to), rank(m.from));
        remember(s, u, captured);
        b[static_cast<size_t>(captured)] = {};
    }
    bool castle = p.type == King && std::abs(file(m.to) - file(m.from)) == 2;
    if (castle) {
        int home = rank(m.from), rookFrom = sq(file(m.to) == 6 ? 7 : 0, home);
        int rookTo = sq(file(m.to) == 6 ? 5 : 3, home);
        remember(s, u, rookFrom); remember(s, u, rookTo);
        b[static_cast<size_t>(rookTo)] = b[static_cast<size_t>(rookFrom)];
        b[static_cast<size_t>(rookFrom)] = {};
    }
    b[static_cast<size_t>(m.to)] = m.promotion ? Piece{m.promotion, p.owner, p.id} : p;
    b[static_cast<size_t>(m.from)] = {};
    if (p.type == King) s.castling &= ~(p.owner == White ? WK | WQ : BK | BQ);
    if (m.from == sq(0,0) || m.to == sq(0,0)) s.castling &= ~WQ;
    if (m.from == sq(7,0) || m.to == sq(7,0)) s.castling &= ~WK;
    if (m.from == sq(0,7) || m.to == sq(0,7)) s.castling &= ~BQ;
    if (m.from == sq(7,7) || m.to == sq(7,7)) s.castling &= ~BK;
    s.epSquare = p.type == Pawn && std::abs(rank(m.to) - rank(m.from)) == 2 ?
                 sq(file(m.from), (rank(m.to) + rank(m.from)) / 2) : -1;
    s.halfmove = (p.type == Pawn || target.type || epCapture) ? 0 :
                 (s.halfmove == std::numeric_limits<int>::max() ? s.halfmove : s.halfmove + 1);
    if (s.turn == Black && s.fullmove < std::numeric_limits<int>::max()) ++s.fullmove;
    s.turn = opponent(s.turn);
    s.result = {};
    return u;
}
inline void unmakeAction(ChessState& s, const UndoInfo& u) {
    for (int i = 0; i < u.count; ++i) s.board.cells[static_cast<size_t>(u.squares[static_cast<size_t>(i)])] = u.before[static_cast<size_t>(i)];
    s.turn = u.turn; s.castling = u.castling; s.epSquare = u.epSquare;
    s.halfmove = u.halfmove; s.fullmove = u.fullmove;
    s.positionHistory.resize(u.historySize); s.result = u.result;
}
inline std::vector<Move> legalMoves(ChessState& s) {
    std::vector<Move> out;
    for (Move m : pseudoMoves(s)) {
        int mover = s.turn;
        UndoInfo u = applyMove(s, m);
        bool safe = !inCheck(s, mover);
        unmakeAction(s, u);
        if (safe) out.push_back(m);
    }
    return out;
}

// The FEN en-passant field records the last double step. Repetition only distinguishes
// it when an en-passant capture is actually legal (including king safety).
inline int effectiveEp(ChessState& s) {
    if (s.epSquare < 0) return -1;
    int r = rank(s.epSquare), fromR = r + (s.turn == White ? -1 : 1);
    for (int df : {-1, 1}) {
        int f = file(s.epSquare) + df;
        if (!inside(f, fromR)) continue;
        int from = sq(f, fromR);
        if (!isPiece(s.board.cells[static_cast<size_t>(from)], Pawn, s.turn)) continue;
        Move m{from, s.epSquare, 0};
        int mover = s.turn;
        UndoInfo u = applyMove(s, m);
        bool legal = !inCheck(s, mover);
        unmakeAction(s, u);
        if (legal) return s.epSquare;
    }
    return -1;
}
inline std::string placement(const ChessState& s) {
    std::string out;
    for (int r = 7; r >= 0; --r) {
        int empty = 0;
        for (int f = 0; f < 8; ++f) {
            Piece p = s.board.cells[static_cast<size_t>(sq(f,r))];
            if (!p.type) { ++empty; continue; }
            if (empty) { out += static_cast<char>('0' + empty); empty = 0; }
            out += pieceChar(p);
        }
        if (empty) out += static_cast<char>('0' + empty);
        if (r) out += '/';
    }
    return out;
}
inline std::string rights(unsigned c) {
    std::string out;
    if (c & WK) out += 'K';
    if (c & WQ) out += 'Q';
    if (c & BK) out += 'k';
    if (c & BQ) out += 'q';
    return out.empty() ? "-" : out;
}
inline std::string fen(const ChessState& s) {
    return placement(s) + (s.turn == White ? " w " : " b ") + rights(s.castling) + " " +
           (s.epSquare < 0 ? "-" : squareName(s.epSquare)) + " " +
           std::to_string(s.halfmove) + " " + std::to_string(s.fullmove);
}
inline std::string positionKey(ChessState& s) {
    int ep = effectiveEp(s);
    return placement(s) + (s.turn == White ? " w " : " b ") + rights(s.castling) + " " +
           (ep < 0 ? "-" : squareName(ep));
}
inline int repetitions(ChessState& s) {
    std::string current = positionKey(s);
    return static_cast<int>(std::count(s.positionHistory.begin(), s.positionHistory.end(), current));
}
// Conservative proven dead positions. Other rare material configurations are left
// ongoing rather than wrongly declared drawn.
inline bool deadPosition(const ChessState& s) {
    int knights = 0, bishops = 0, bishopColor = -1;
    for (int i = 0; i < 64; ++i) {
        Piece p = s.board.cells[static_cast<size_t>(i)];
        if (!p.type || p.type == King) continue;
        if (p.type == Knight) { ++knights; continue; }
        if (p.type == Bishop) {
            ++bishops;
            int color = (file(i) + rank(i)) & 1;
            if (bishopColor == -1) bishopColor = color;
            else if (bishopColor != color) bishopColor = 2;
            continue;
        }
        return false;
    }
    return (knights == 0 && (bishops == 0 || bishopColor != 2)) || (knights == 1 && bishops == 0);
}
inline GameResult outcome(ChessState& s) {
    if (s.result.kind != ResultKind::Ongoing) return s.result;
    if (legalMoves(s).empty()) return inCheck(s, s.turn) ? GameResult{ResultKind::Win, opponent(s.turn), Checkmate} :
                                                        GameResult{ResultKind::Draw, -1, Stalemate};
    if (deadPosition(s)) return {ResultKind::Draw, -1, DeadPosition};
    if (repetitions(s) >= 5) return {ResultKind::Draw, -1, Fivefold};
    if (s.halfmove >= 150) return {ResultKind::Draw, -1, SeventyFiveMove};
    return {};
}
inline bool claimThreefold(ChessState& s) { return repetitions(s) >= 3; }
inline bool claimFiftyMove(const ChessState& s) { return s.halfmove >= 100; }

// Validates a complete FEN before replacing the caller's state. FEN cannot reconstruct
// earlier repetition history, so the loaded position becomes the sole known entry.
inline bool parseInteger(std::string_view text, int minimum, int& value) {
    long long temp = 0;
    auto [end, error] = std::from_chars(text.data(), text.data() + text.size(), temp);
    if (error != std::errc{} || end != text.data() + text.size() || temp < minimum || temp > std::numeric_limits<int>::max()) return false;
    value = static_cast<int>(temp); return true;
}
inline bool loadFen(ChessState& destination, const std::string& input, std::string& error) {
    std::istringstream stream(input);
    std::array<std::string, 6> fields;
    std::string extra;
    for (auto& field : fields) if (!(stream >> field)) { error = "FEN requires six fields"; return false; }
    if (stream >> extra) { error = "FEN has extra fields"; return false; }
    ChessState s;
    int r = 7, f = 0, slashes = 0, kings[2] = {0, 0}, nextPieceId = 1;
    for (char c : fields[0]) {
        if (c == '/') {
            if (f != 8 || r == 0) { error = "invalid FEN rank"; return false; }
            --r; f = 0; ++slashes; continue;
        }
        if (c >= '1' && c <= '8') { f += c - '0'; if (f > 8) { error = "rank too wide"; return false; } continue; }
        int type = pieceType(c);
        if (!type || f >= 8) { error = "invalid FEN piece or rank"; return false; }
        int side = std::isupper(static_cast<unsigned char>(c)) ? White : Black;
        if (type == Pawn && (r == 0 || r == 7)) { error = "pawn on back rank"; return false; }
        s.board.cells[static_cast<size_t>(sq(f++, r))] = {type, side, nextPieceId++};
        if (type == King) ++kings[side];
    }
    if (slashes != 7 || f != 8 || kings[0] != 1 || kings[1] != 1) { error = "invalid board or king count"; return false; }
    if (fields[1] == "w") s.turn = White;
    else if (fields[1] == "b") s.turn = Black;
    else { error = "invalid side to move"; return false; }
    if (fields[2] != "-") {
        for (char c : fields[2]) {
            unsigned bit = c == 'K' ? WK : c == 'Q' ? WQ : c == 'k' ? BK : c == 'q' ? BQ : 0;
            if (!bit || (s.castling & bit)) { error = "invalid castling rights"; return false; }
            s.castling |= bit;
        }
    }
    const auto& b = s.board.cells;
    for (auto [bit, k, rook, side] : {std::tuple{WK, sq(4,0), sq(7,0), White},
                                     std::tuple{WQ, sq(4,0), sq(0,0), White},
                                     std::tuple{BK, sq(4,7), sq(7,7), Black},
                                     std::tuple{BQ, sq(4,7), sq(0,7), Black}}) {
        if ((s.castling & bit) && (!isPiece(b[static_cast<size_t>(k)], King, side) ||
                                   !isPiece(b[static_cast<size_t>(rook)], Rook, side))) {
            error = "castling right without home king and rook"; return false;
        }
    }
    if (fields[3] != "-") {
        s.epSquare = parseSquare(fields[3]);
        if (s.epSquare < 0 || rank(s.epSquare) != (s.turn == White ? 5 : 2) ||
            b[static_cast<size_t>(s.epSquare)].type ||
            !isPiece(b[static_cast<size_t>(s.epSquare + (s.turn == White ? -8 : 8))], Pawn, opponent(s.turn))) {
            error = "invalid en-passant square"; return false;
        }
    }
    if (!parseInteger(fields[4], 0, s.halfmove) || !parseInteger(fields[5], 1, s.fullmove)) {
        error = "invalid move counters"; return false;
    }
    if (inCheck(s, opponent(s.turn))) { error = "side that just moved remains in check"; return false; }
    s.positionHistory.push_back(positionKey(s));
    s.result = outcome(s);
    destination = std::move(s);
    return true;
}
inline bool makeAction(ChessState& s, const Action& action, UndoInfo& undo) {
    if (s.result.kind != ResultKind::Ongoing) return false;
    if (action.kind == Core::ActionKind::RuleCommand) {
        ResultKind end = ResultKind::Ongoing;
        int reason = None;
        int winner = -1;
        switch (action.command) {
            case ClaimThreefold:
                if (claimThreefold(s)) { end = ResultKind::Draw; reason = ThreefoldClaim; }
                break;
            case ClaimFiftyMove:
                if (claimFiftyMove(s)) { end = ResultKind::Draw; reason = FiftyMoveClaim; }
                break;
            case AgreeDraw: end = ResultKind::Draw; reason = Agreement; break;
            case Resign: end = ResultKind::Win; reason = Resignation; winner = opponent(s.turn); break;
            default: break;
        }
        if (end == ResultKind::Ongoing) return false;
        undo = {};
        undo.turn = s.turn; undo.castling = s.castling; undo.epSquare = s.epSquare;
        undo.halfmove = s.halfmove; undo.fullmove = s.fullmove;
        undo.historySize = s.positionHistory.size(); undo.result = s.result;
        s.result = {end, winner, reason};
        return true;
    }
    if (action.kind != Core::ActionKind::Move) return false;
    auto moves = legalMoves(s);
    if (std::find(moves.begin(), moves.end(), action.move) == moves.end()) return false;
    undo = applyMove(s, action.move);
    s.positionHistory.push_back(positionKey(s));
    s.result = outcome(s);
    return true;
}
inline uint64_t perft(ChessState& s, int depth) {
    if (depth == 0) return 1;
    auto moves = legalMoves(s);
    if (depth == 1) return moves.size();
    uint64_t nodes = 0;
    for (Move m : moves) {
        UndoInfo u = applyMove(s, m);
        nodes += perft(s, depth - 1);
        unmakeAction(s, u);
    }
    return nodes;
}
// Standard Chess is one replaceable rule module, including its own definition
// schema, initial position, board boundary, and victory evaluation.
struct Rules {
    using State = ChessState;
    static const std::array<PieceDefinition, 6>& pieceDefinitions() {
        return ChessPieces;
    }
    static Board boardBoundary() { return Board(8,8); }
    static State initialPosition() {
        State s; std::string error;
        if (!loadFen(s,std::string(StartFen),error)) throw std::logic_error(error);
        return s;
    }
    static GameResult victoryCondition(State& s) { return outcome(s); }
};
static_assert(Core::RuleModule<Rules>);
} // namespace StandardChess
