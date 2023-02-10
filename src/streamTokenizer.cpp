#include <iostream>
#include <fstream>
#include <sstream>

std::ifstream inputFile;

int lineNum = 0;
int lineOff = 0;

std::string getToken(){
    std::string line;
    std::string token;

    if (std::getline(inputFile, line)) {
        
    }
    return token;
}

int main()
{
    std::string line;

    // std::ifstream file("test.txt");
    inputFile.open("../lab1_assign/input-1");
    while (std::getline(inputFile, line))
    {
        lineNum++;
        lineOff = 0;

        std::istringstream linestream(line);
        std::string token;

        while (linestream >> token)
        {
            lineOff = line.find(token, lineOff);
            lineOff++;
            std::cout << "Line: " << std::setw(3) << lineNum << " Offset: " << std::setw(3) << lineOff << " Token: " << token << std::endl;
            lineOff += token.length();
        }
    }

    std::cout << "EOF at line " << lineNum << " and line offset " << lineOff << std::endl;
    return 0;
}
