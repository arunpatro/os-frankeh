#include <cstdio>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <map>
#include <string>
#include <vector>

//// CONSTANTS
const int MACHINE_SIZE = 512;
const int MAX_SYMBOL_LENGTH = 16;
const int MAX_SYMBOLS_PER_MODULE = 16;
const int MAX_USES_PER_MODULE = 16;
const int MAX_NUM = int(2E+30);

class Symbol {
   public:
    std::string name;
    int relAddr;
    int absAddr;
    int moduleNum;
    bool used;
    bool redefined;
    Symbol(std::string name = "", int value = 0, int moduleNum = 0) {
        name = name;
        relAddr = value;
        used = false;
        moduleNum = moduleNum;
        redefined = false;
    }
};

struct Token {
    std::string val;
    int lineNum;
    int lineOff;
};

std::map<std::string, Symbol> symbolTable;
std::vector<std::string> symbolTableOrder;

std::ifstream inputFile;
int lineNum = 0;
int lineOff = 0;
// int lastLineOff = 1;
int fileFinalPosition = 0;

void __parseerror(int errcode, int lineNum, int lineOff) {
    static std::string errstr[] = {
        "NUM_EXPECTED",            // Number expect, anything >= 2^30 is not a number either
        "SYM_EXPECTED",            // Symbol Expected
        "ADDR_EXPECTED",           // Addressing Expected which is A/E/I/R
        "SYM_TOO_LONG",            // Symbol Name is too long
        "TOO_MANY_DEF_IN_MODULE",  // > 16
        "TOO_MANY_USE_IN_MODULE",  // > 16
        "TOO_MANY_INSTR",          // total num_instr exceeds memory size (512)
    };
    std::printf("Parse Error line %d offset %d: %s\n", lineNum, lineOff, errstr[errcode].c_str());
    exit(errcode);
}

std::string line;
char* tokenChars;

Token getToken() {
    Token tokenStruct;
    bool prevTokenExists;
    prevTokenExists = tokenChars != NULL;

    // if we already have a token, then we can get the next token with strtok
    if (prevTokenExists) {
        tokenChars = std::strtok(NULL, " \t\n");
    }

    // if token doesn't exist, get a new token
    while (!tokenChars) {
        // if we get a new line, then increment lineNum and get token from that line
        if (std::getline(inputFile, line)) {
            lineNum++;
            lineOff = 0;
            fileFinalPosition = 1;
            tokenChars = std::strtok(&line[0], " \t\n");
            // if token exists, then obv while loop breaks
        } else {
            // we don't have a token and we don't have a new line, so we are at the end of the file
            // return empty token
            break;
        }
    }

    if (tokenChars) {
        // lineOff = line.find(tokenChars, lineOff);
        lineOff = tokenChars - const_cast<char*>(line.c_str());

        if (inputFile.eof()) {
            // end of file in this line, so final offset is end of line
            
                fileFinalPosition = line.length();
        } else {
            // next line we read will have eof
            fileFinalPosition = line.length() + 1;
        }

        tokenStruct.val = tokenChars;
        tokenStruct.lineNum = lineNum;
        tokenStruct.lineOff = lineOff + 1;

        // lineOff += tokenStruct.val.length(); // this gives some errors at end of line
    } else {
        // no token, so we are either on a line with end of file, or we are at the end of the file
        tokenStruct.val = "";
        tokenStruct.lineOff = fileFinalPosition;
        tokenStruct.lineNum = lineNum;
    }

    return tokenStruct;
}

int readInt() {
    int number;
    Token token = getToken();
    try {
        number = std::stoi(token.val);
        if (number > MAX_NUM) {
            __parseerror(0, token.lineNum, token.lineOff);
        }
    } catch (const std::exception& e) {
        __parseerror(0, token.lineNum, token.lineOff);
    }
    return number;
}

std::string readSymbol() {
    Token token = getToken();
    std::string symbol = token.val;
    // print error if symbol name is too long
    if (symbol.length() > MAX_SYMBOL_LENGTH) {
        __parseerror(3, token.lineNum, token.lineOff);
    } else if (symbol.length() == 0) {
        // empty string implies EOF, so no symbol
        // if lineOff == 1, then it is a new line, so lineOff is actually lastLineOff and lineNum is lineNum - 1
        __parseerror(1, lineNum, fileFinalPosition);
    } else if (!isalpha(symbol[0])) {
        __parseerror(1, token.lineNum, token.lineOff);
    } else {
        for (int i = 1; i < symbol.length(); i++) {
            if (!isalnum(symbol[i])) {
                __parseerror(1, token.lineNum, token.lineOff);
            }
        }
    }
    return symbol;
}

char readIEAR() {
    Token token = getToken();

    // check if token is empty
    if (token.val.length() == 0) {
        // empty string implies EOF, so no symbol
        // if lineOff == 1, then it is a new line, so lineOff is actually lastLineOff and lineNum is lineNum - 1
        __parseerror(2, token.lineNum, token.lineOff);
    }
    char addressmode = token.val[0];

    // check if addressmode is valid
    if (addressmode != 'I' && addressmode != 'E' && addressmode != 'A' && addressmode != 'R') {
        __parseerror(2, token.lineNum, token.lineOff);
    }
    return addressmode;
}

// 1. verify syntax
// 2. determine base addr of each module
// 3. absolute addr of each symbol
void pass1() {
    Token token;
    int baseAddr = 0;
    int totalInstructionCount = 0;
    int moduleNum = 0;

    while (!inputFile.eof() && inputFile.peek() != EOF) {
        token = getToken();
        if (token.val.length() == 0) {
            // empty string implies EOF, so no symbol
            break;
        }

        // DEFCOUNT CHECKS
        int defcount = std::stoi(token.val);
        // print error if defcount > 16
        if (defcount > MAX_SYMBOLS_PER_MODULE) {
            __parseerror(4, token.lineNum, token.lineOff);
        } else if (defcount < 0) {
            __parseerror(0, token.lineNum, token.lineOff);
        }

        // increment module number now only after successfull defcount
        moduleNum++;
        baseAddr = totalInstructionCount;

        // START READING SYMBOLS for DEFCOUNT
        for (int i = 0; i < defcount; i++) {
            std::string symbolName = readSymbol();

            int relAddr = readInt();
            if (symbolTable.count(symbolName) != 0) {
                symbolTable[symbolName].redefined = true;
                // print warning if symbol is redefined
                std::printf("Warning: Module %d: %s redefined and ignored\n", moduleNum, symbolName.c_str());

            } else {
                Symbol symbol = Symbol(symbolName, relAddr);
                symbol.absAddr = baseAddr + relAddr;
                symbol.moduleNum = moduleNum;
                symbolTable[symbolName] = symbol;
                symbolTableOrder.push_back(symbolName);
            }
        }

        token = getToken();
        int usecount = std::stoi(token.val);
        // print error if usecount > 16
        if (usecount > MAX_USES_PER_MODULE) {
            __parseerror(5, token.lineNum, token.lineOff);
        }

        for (int i = 0; i < usecount; i++) {
            std::string symbol = readSymbol();
        }

        token = getToken();
        int codecount = std::stoi(token.val);
        // print warning if total instruction count exceeds 512
        if (totalInstructionCount + codecount > MACHINE_SIZE) {
            __parseerror(6, token.lineNum, token.lineOff);
        }
        for (int i = 0; i < codecount; i++) {
            char addressmode = readIEAR();
            int instruction = readInt();
            int opcode = instruction / 1000;
            int operand = instruction % 1000;
        }

        // relative addr of each symbol cannot exceed the size of the module
        for (auto it = symbolTable.begin(); it != symbolTable.end(); it++) {
            // for all symbols that are defined in this module, and have relative addr >= module size
            if (it->second.moduleNum == moduleNum && it->second.relAddr >= codecount) {
                std::printf("Warning: Module %d: %s too big %d (max=%d) assume zero relative\n", moduleNum, it->first.c_str(), it->second.relAddr, codecount - 1);
                it->second.relAddr = 0;
                it->second.absAddr = baseAddr;
            }
        }

        totalInstructionCount += codecount;
    }

    // print the symbol table
    std::cout << "Symbol Table" << std::endl;
    std::string errorMsg = "";
    for (auto it = symbolTableOrder.begin(); it != symbolTableOrder.end(); it++) {
        if (symbolTable[*it].redefined) {
            errorMsg = " Error: This variable is multiple times defined; first value used";
        } else {
            errorMsg = "";
        }
        std::cout << *it << "=" << symbolTable[*it].absAddr << errorMsg << std::endl;
    }

    // for (auto it = symbolTable.begin(); it != symbolTable.end(); it++) {
    //     if (it->second.redefined) {
    //         errorMsg = " Error: This variable is multiple times defined; first value used";
    //     } else {
    //         errorMsg = "";
    //     }
    //     std::cout << it->first << "=" << it->second.absAddr << errorMsg << std::endl;
    // }
}

void pass2() {
    std::cout << std::endl
              << "Memory Map" << std::endl;
    int baseAddr = 0;
    int totalInstructionCount = 0;
    std::string errorMsg = "";
    int modifiedInstruction;
    int moduleNum = 0;
    while (!inputFile.eof() && inputFile.peek() != EOF) {
        baseAddr = totalInstructionCount;
        Token token = getToken();
        if (token.val.length() == 0) {
            // empty string implies EOF, so no symbol
            break;
        }
        int defcount = std::stoi(token.val);
        // increment module number now only after successfull defcount
        moduleNum++;
        baseAddr = totalInstructionCount;
        for (int i = 0; i < defcount; i++) {
            std::string symbol = readSymbol();
            int relAddr = readInt();
        }

        int usecount = readInt();
        std::vector<std::pair<std::string, bool> > useList;
        for (int i = 0; i < usecount; i++) {
            std::string symbol = readSymbol();
            useList.push_back(std::make_pair(symbol, false));
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
                std::string errorMsg = "";
                if (opcode >= 10) {
                    errorMsg = " Error: Illegal opcode; treated as 9999";
                    modifiedInstruction = 9999;
                } else if (operand >= usecount) {
                    errorMsg = " Error: External address exceeds length of uselist; treated as immediate";
                    modifiedInstruction = instruction;
                } else {
                    std::string symbol = useList[operand].first;

                    // mark the symbol as used from the use list
                    useList[operand].second = true;

                    if (symbolTable.count(symbol) == 0) {
                        errorMsg = " Error: " + symbol + " is not defined; zero used";
                        modifiedInstruction = (opcode * 1000);
                    } else {
                        symbolTable[symbol].used = true;
                        int symbolAddr = symbolTable[symbol].absAddr;
                        modifiedInstruction = (opcode * 1000) + symbolAddr;
                        errorMsg = "";
                    }
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
                } else if (operand >= codecount) {
                    errorMsg = " Error: Relative address exceeds module size; zero used";
                    modifiedInstruction = (opcode * 1000) + baseAddr;
                } else {
                    modifiedInstruction = (opcode * 1000) + (baseAddr + operand);
                    errorMsg = "";
                }
                std::cout << std::setfill('0') << std::setw(3) << addr << ": " << std::setw(4) << modifiedInstruction << errorMsg << std::endl;
            }
        }

        // print unused symbol warnings in the use list
        for (auto it = useList.begin(); it != useList.end(); it++) {
            if (!it->second) {
                std::printf("Warning: Module %d: %s appeared in the uselist but was not actually used\n", moduleNum, it->first.c_str());
            }
        }

        totalInstructionCount += codecount;
    }

    // print unused symbol warnings
    for (auto it = symbolTable.begin(); it != symbolTable.end(); it++) {
        if (!it->second.used) {
            std::printf("Warning: Module %d: %s was defined but never used\n", it->second.moduleNum, it->first.c_str());
        }
    }
}

void printTokenize() {
    while (true) {
        Token token = getToken();
        if (token.val == "") {
            break;
        }
        // std::cout << "Line: " << std::setw(3) << token.lineNum << " Offset: " << std::setw(3) << token.lineOff << " Token: " << token.val << std::endl;
        std::printf("Token: %d:%d : %s\n", token.lineNum, token.lineOff, token.val.c_str());
    }

    // std::cout << "EOF at line " << lineNum << " and line offset " << lineOff << std::endl;
    std::printf("Final Spot in File : line=%d offset=:%d\n", lineNum, fileFinalPosition);
}

int main(int argc, char* argv[]) {
    // inputFile.open(argv[1]);
    // printTokenize();
    // inputFile.close();

    try {
        inputFile.open(argv[1]);
    } catch (std::exception& e) {
        std::cout << "Not a valid inputfile <" << argv[1] << "" << std::endl;
        return 1;
    }

    pass1();
    inputFile.close();

    lineNum = 0;
    lineOff = 0;
    inputFile.open(argv[1]);
    pass2();
    inputFile.close();

    return 0;
}
