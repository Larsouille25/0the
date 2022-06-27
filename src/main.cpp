#include <iostream>
#include <lwlogger/logger.hpp>

int main()
{
    Larsouille::Logger logger("logs/");
    logger.info("Hello world !");

    return 0;
}