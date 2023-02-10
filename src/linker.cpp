#include <fstream>
#include <iostream>
#include <map>
#include <sstream>
#include <vector>

struct Token {
    std::string val;
    int lineNum;
    int lineOff;
};

std::map<std::string, int> symbolTable;

std::ifstream inputFile;
int lineNum = 1;
int lineOff = 1;

Token getToken() {
    Token token;
    char c;

    while (inputFile.get(c)) {
        if (c == ' ' || c == '\t') {
            if (token.val.length() > 0) {
                token.lineOff = lineOff - token.val.length();
                token.lineNum = lineNum;
                lineOff++;
                break;
            }
            lineOff++;
        } else if (c == '\n') {
            if (token.val.length() > 0) {
                token.lineOff = lineOff - token.val.length();
                token.lineNum = lineNum;
                lineNum++;
                lineOff = 1;
                break;
            }
            lineOff = 1;
            lineNum++;
        } else {
            token.val += c;
            lineOff++;
        }
    }

    if (inputFile.eof() && token.val.length() > 0) {
        token.lineOff = lineOff - token.val.length();
        token.lineNum = lineNum;
    }

    return token;
}

int readInt() {
    Token token = getToken();
    int number = std::stoi(token.val);
    return number;
}

std::string readSymbol() {
    Token token = getToken();
    std::string symbol = token.val;
    return symbol;
}

char readIEAR() {
    Token token = getToken();
    char addressmode = token.val[0];
    return addressmode;
}

// 1. verify syntax
// 2. determine base addr of each module
// 3. absolute addr of each symbol
void pass1() {
    // int baseAddr = 0;
    int totalInstructionCount = 0;
    while (!inputFile.eof() && inputFile.peek() != EOF) {
        int defcount = readInt();
        for (int i = 0; i < defcount; i++) {
            std::string symbol = readSymbol();
            int addr = readInt();
            symbolTable[symbol] = totalInstructionCount + addr;
        }

        int usecount = readInt();
        for (int i = 0; i < usecount; i++) {
            std::string symbol = readSymbol();
        }

        int codecount = readInt();
        for (int i = 0; i < codecount; i++) {
            char addressmode = readIEAR();
            int instruction = readInt();
            int opcode = instruction / 1000;
            int operand = instruction % 1000;
        }

        totalInstructionCount += codecount;
    }

    // print the symbol table
    std::cout << "Symbol Table" << std::endl;
    for (auto it = symbolTable.begin(); it != symbolTable.end(); it++) {
        std::cout << it->first << "=" << it->second << std::endl;
    }
}

void printAllTokens() {
    while (!inputFile.eof()) {
        Token token = getToken();
        if (token.val.length() == 0) {
            break;
        }
        std::cout << "Line: " << std::setw(3) << token.lineNum << " Offset: " << std::setw(3) << token.lineOff << " Token: " << token.val << std::endl;
    }

    std::cout << "EOF at line " << lineNum << " and line offset " << lineOff << std::endl;
}

int main(int argc, char* argv[]) {
    // inputFile.open(argv[1]);
    inputFile.open("../lab1_assign/input-1");
    // inputFile.open("test.txt");

    // printAllTokens();
    pass1();

}
