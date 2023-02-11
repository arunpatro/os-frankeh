#include <fstream>
#include <iostream>
#include <map>
#include <vector>
#include <iomanip>

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
    int baseAddr = 0;
    int totalInstructionCount = 0;
    while (!inputFile.eof() && inputFile.peek() != EOF) {
        baseAddr = totalInstructionCount;
        int defcount = readInt();
        for (int i = 0; i < defcount; i++) {
            std::string symbol = readSymbol();
            int relAddr = readInt();
            symbolTable[symbol] = baseAddr + relAddr;
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

void pass2() {
    std::cout << std::endl
              << "Memory Map" << std::endl;
    int baseAddr = 0;
    int totalInstructionCount = 0;
    while (!inputFile.eof() && inputFile.peek() != EOF) {
        baseAddr = totalInstructionCount;
        int defcount = readInt();
        for (int i = 0; i < defcount; i++) {
            std::string symbol = readSymbol();
            int relAddr = readInt();
        }

        int usecount = readInt();
        std::vector<std::string> useList;
        for (int i = 0; i < usecount; i++) {
            std::string symbol = readSymbol();
            useList.push_back(symbol);
        }

        int codecount = readInt();
        for (int i = 0; i < codecount; i++) {
            char addressmode = readIEAR();
            int instruction = readInt();
            int opcode = instruction / 1000;
            int operand = instruction % 1000;

            int addr = baseAddr + i;

            if (addressmode == 'I') {
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << instruction << std::endl;
            } else if (addressmode == 'E') {
                std::string symbol = useList[operand];
                int symbolAddr = symbolTable[symbol];
                int modifiedInstruction = (opcode * 1000) + symbolAddr;
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << std::endl;
            } else if (addressmode == 'A') {
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << instruction << std::endl;
            } else if (addressmode == 'R') {
                int modifiedInstruction = (opcode * 1000) + (baseAddr + operand);
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << std::endl;
            }
        }

        totalInstructionCount += codecount;
    }
}

void printAllTokens() {
    while (!inputFile.eof() && inputFile.peek() != EOF) {
        Token token = getToken();
        std::cout << "Line: " << std::setw(3) << token.lineNum << " Offset: " << std::setw(3) << token.lineOff << " Token: " << token.val << std::endl;
    }

    std::cout << "EOF at line " << lineNum << " and line offset " << lineOff << std::endl;
}

int main(int argc, char* argv[]) {
    // inputFile.open(argv[1]);
    // inputFile.open("../lab1_assign/input-1");
    // printAllTokens();
    // inputFile.close();

    inputFile.open(argv[1]);
    pass1();
    inputFile.close();

    // inputFile.open("../lab1_assign/input-1");
    inputFile.open(argv[1]);
    pass2();
    inputFile.close();
}
