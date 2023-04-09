build:
	mkdir -p ./bin
	GOBIN="`pwd`/bin" go install ./...
copy:
	mkdir -p ../maelstrom/solutions/go
	cp ./bin/* ../maelstrom/solutions/go

all: build copy
