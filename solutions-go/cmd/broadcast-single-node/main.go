package main

import (
	"encoding/json"
	"fmt"
	maelstrom "github.com/jepsen-io/maelstrom/demo/go"
	"os"
	"sync"
)

type BroadcastMessage struct {
	Type    string `json:"type"`
	Message uint64 `json:"message"`
}

type TopologyMessage struct {
	Type     string              `json:"type"`
	Topology map[string][]string `json:"topology"`
}

type ReadOkMessage struct {
	Type     string   `json:"type"`
	Messages []uint64 `json:"messages"`
}

type EmptyOkMessage struct {
	Type string `json:"type"`
}

type Set = map[uint64]struct{}
type void struct{}

var member void

type State struct {
	Neighbors []string
	Messages  Set
	mu        sync.Mutex
}

func startNode(node *maelstrom.Node) {
	if err := node.Run(); err != nil {
		fmt.Printf("ERROR: %s", err)
		os.Exit(1)
	}
}

func main() {
	node := maelstrom.NewNode()
	defer startNode(node)

	state := State{
		Neighbors: make([]string, 0),
		Messages:  make(Set),
	}

	node.Handle("topology", func(msg maelstrom.Message) error {
		var message TopologyMessage
		if err := json.Unmarshal(msg.Body, &message); err != nil {
			return err
		}
		state.mu.Lock()
		state.Neighbors = message.Topology[node.ID()]
		state.mu.Unlock()
		return node.Reply(msg, EmptyOkMessage{Type: "topology_ok"})
	})

	node.Handle("broadcast", func(msg maelstrom.Message) error {
		var message BroadcastMessage
		if err := json.Unmarshal(msg.Body, &message); err != nil {
			return err
		}
		state.mu.Lock()
		state.Messages[message.Message] = member
		state.mu.Unlock()

		return node.Reply(msg, EmptyOkMessage{Type: "broadcast_ok"})
	})

	node.Handle("read", func(msg maelstrom.Message) error {
		var message map[string]any
		if err := json.Unmarshal(msg.Body, &message); err != nil {
			return err
		}
		var payload ReadOkMessage
		payload.Type = "read_ok"
		state.mu.Lock()
		payload.Messages = make([]uint64, 0)
		for m := range state.Messages {
			payload.Messages = append(payload.Messages, m)
		}
		state.mu.Unlock()
		return node.Reply(msg, payload)
	})
}
