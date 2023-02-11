all: clean linker run grade

linker: linker.cpp
	g++ -g linker.cpp -o linker

clean:
	rm -f linker *~

run:
	cd ../lab1_assign/; ./runit.sh ../outputs ../src/linker

grade:
	../lab1_assign/gradeit.sh ../lab1_assign/ ../outputs