#include <fstream>
#include <iomanip>
#include <iostream>
#include <map>
#include <vector>

//// CONSTANTS
const int MACHINE_SIZE = 512;

class Symbol {
   public:
    std::string name;
    int relAddr;
    int absAddr;
    // bool used;
    // bool defined;
    bool redefined;
    // bool error;
    // bool warning;
    Symbol(std::string name = "", int value = 0) {
        name = name;
        relAddr = value;
        // used = false;
        // defined = false;
        redefined = false;
        // error = false;
        // warning = false;
    }
};

struct Token {
    std::string val;
    int lineNum;
    int lineOff;
};

std::map<std::string, Symbol> symbolTable;

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
            std::string symbolName = readSymbol();
            int relAddr = readInt();

            if (symbolTable.count(symbolName) != 0) {
                symbolTable[symbolName].redefined = true;
            } else {
                Symbol symbol = Symbol(symbolName, relAddr);
                symbol.absAddr = baseAddr + relAddr;
                symbolTable[symbolName] = symbol;
            }
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
    std::string errorMsg = "";
    for (auto it = symbolTable.begin(); it != symbolTable.end(); it++) {
        if (it->second.redefined) {
            errorMsg = " Error: This variable is multiple times defined; first value used";
        } else {
            errorMsg = "";
        }
        std::cout << it->first << "=" << it->second.absAddr << errorMsg << std::endl;
    }
}

void pass2() {
    std::cout << std::endl
              << "Memory Map" << std::endl;
    int baseAddr = 0;
    int totalInstructionCount = 0;
    std::string errorMsg = "";
    int modifiedInstruction;
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
                if (instruction >= 10000) {
                    errorMsg = " Error: Illegal immediate value; treated as 9999";
                    modifiedInstruction = 9999;
                } else {
                    modifiedInstruction = instruction;
                    errorMsg = "";
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            } else if (addressmode == 'E') {
                std::string symbol = useList[operand];
                int symbolAddr = symbolTable[symbol].absAddr;

                std::string errorMsg = "";
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                } else if (operand >= usecount) {
                    errorMsg = " Error: External address exceeds length of uselist; treated as immediate";
                    modifiedInstruction = instruction;
                } else if (symbolTable.count(symbol) == 0) {
                    errorMsg = " Error: " + symbol + " is not defined; zero used";
                    modifiedInstruction = (opcode * 1000);
                } else {
                    modifiedInstruction = (opcode * 1000) + symbolAddr;
                    errorMsg = "";
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            } else if (addressmode == 'A') {
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                } else if (operand >= MACHINE_SIZE) {
                    errorMsg = " Error: Absolute address exceeds machine size; zero used";
                    modifiedInstruction = (opcode * 1000);
                } else {
                    modifiedInstruction = instruction;
                    errorMsg = "";
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            } else if (addressmode == 'R') {
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                } else {
                    modifiedInstruction = (opcode * 1000) + (baseAddr + operand);
                    errorMsg = "";
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
