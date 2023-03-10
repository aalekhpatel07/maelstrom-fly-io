package main

import (
	"encoding/json"
	"fmt"
	maelstrom "github.com/jepsen-io/maelstrom/demo/go"
	"os"
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
		state.Neighbors = message.Topology[node.ID()]
		return node.Reply(msg, EmptyOkMessage{Type: "topology_ok"})
	})

	node.Handle("broadcast", func(msg maelstrom.Message) error {
		var message BroadcastMessage
		if err := json.Unmarshal(msg.Body, &message); err != nil {
			return err
		}

		_, ok := state.Messages[message.Message]

		if ok {
			return node.Reply(msg, EmptyOkMessage{Type: "broadcast_ok"})
		}

		state.Messages[message.Message] = member

		for _, neighbor := range state.Neighbors {
			if neighbor != msg.Src {

				var payload BroadcastMessage
				payload.Type = "broadcast"
				payload.Message = message.Message

				if err := node.Send(neighbor, payload); err != nil {
					fmt.Printf("Error sending payload to neighbor %s: %s", neighbor, err)
					os.Exit(1)
				}
			}
		}
		return node.Reply(msg, EmptyOkMessage{Type: "broadcast_ok"})
	})

	node.Handle("read", func(msg maelstrom.Message) error {
		var message map[string]any
		if err := json.Unmarshal(msg.Body, &message); err != nil {
			return err
		}
		payload := ReadOkMessage{
			Type:     "read_ok",
			Messages: make([]uint64, 0),
		}
		for m := range state.Messages {
			payload.Messages = append(payload.Messages, m)
		}
		return node.Reply(msg, payload)
	})

	node.Handle("broadcast_ok", func(msg maelstrom.Message) error {
		return nil
	})
}
