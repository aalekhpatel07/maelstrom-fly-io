package main

import (
	"encoding/json"
	"fmt"
	maelstrom "github.com/jepsen-io/maelstrom/demo/go"
	"os"
	"sync/atomic"
)

type State struct {
	Counter uint64
	Node    *maelstrom.Node
}

func main() {
	node := maelstrom.NewNode()
	state := State{
		Counter: 0,
		Node:    node,
	}

	state.Node.Handle("generate", func(msg maelstrom.Message) error {
		var body map[string]any
		if err := json.Unmarshal(msg.Body, &body); err != nil {
			return err
		}
		atomic.AddUint64(&state.Counter, 1)
		body["type"] = "generate_ok"
		body["id"] = fmt.Sprintf("%s-%4d", state.Node.ID(), state.Counter)
		return state.Node.Reply(msg, body)
	})

	if err := state.Node.Run(); err != nil {
		fmt.Printf("ERROR: %s", err)
		os.Exit(1)
	}
}
