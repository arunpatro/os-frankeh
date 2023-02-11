#include <fstream>
#include <iostream>
#include <map>
#include <vector>
#include <iomanip>


/*
ERROR CODES
1. You should stop processing if a syntax error is detected in the input, print a syntax error message with the line number and the character offset in the input file where observed. A syntax error is defined as a missing token (e.g. 4 used symbols are defined but only 3 are given) or an unexpected token. Stop processing and exit.
2. If a symbol is defined multiple times, print an error message and use the value given in the first definition. The error message is to appear as part of printing the symbol table (following symbol=value printout on the same line).
3. If a symbol is used in an E-instruction but not defined anywhere, print an error message and use the value absolute zero.
4. If a symbol is defined but not used, print a warning message and continue.
5. If an address appearing in a first definition of a symbol exceeds the size of the module, print a warning message in pass1 after processing the module and treat the address given as 0 (relative to the module). If the symbol is a redefinition then print a warning message (see message below).
6. If an external address is too large to reference an entry in the use list, print an error message and treat the address as immediate.
7. If a symbol appears in a use list but is not actually used in the module (i.e., not referred to in an E-type address), print a warning message and continue.
8. If an absolute address exceeds the size of the machine, print an error message and use the absolute value zero.
9. If a relative address exceeds the size of the module, print an error message and use the module relative value zero (that means you still need to remap “0” that to the correct absolute address).
10. If an illegal immediate value (I) is encountered (i.e. >= 10000), print an error and convert the value to 9999.
11. If an illegal opcode is encountered (i.e. op >= 10), print an error and convert the <opcode,operand> to 9999.

Todo:
1
2
3
4
5
6
7
9
10
11

*/

//// CONSTANTS
const int MACHINE_SIZE = 512;

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
                std::string errorMsg = "";
                if (instruction >= 10000) {
                    errorMsg = " Error: Illegal immediate value; treated as 9999";
                    instruction = 9999;
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << instruction << errorMsg << std::endl;
            } else if (addressmode == 'E') {
                std::string symbol = useList[operand];
                int symbolAddr = symbolTable[symbol];
                int modifiedInstruction = (opcode * 1000) + symbolAddr;

                std::string errorMsg = "";
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            } else if (addressmode == 'A') {
                int modifiedInstruction = instruction;
                std::string errorMsg = "";
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                } else if (operand >= MACHINE_SIZE) {
                    errorMsg = " Error: Absolute address exceeds machine size; zero used";
                    modifiedInstruction = (opcode * 1000);
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            } else if (addressmode == 'R') {
                int modifiedInstruction = (opcode * 1000) + (baseAddr + operand);
                
                std::string errorMsg = "";
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
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
