package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
)

type Rem struct {
	Todos []Todo `json:"todos"`
}

type Todo struct {
	Content string `json:"content"`
	Done    bool   `json:"done"`
}

func main() {
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalln("Could not get home directory")
	}
	file_location := fmt.Sprintf("%v/.local/share/rem.json", home)
	// log.Println(file_location)

	file, err := os.OpenFile(file_location, os.O_RDWR, 0644)
	// file, err := os.Open(file_location)
	if err != nil {
		// file probably does not exist, so create it
		new_file, err := os.Create(file_location)
		if err != nil {
			log.Fatalln(err)
		}
		defer new_file.Close()
		new_rem := Rem{
			make([]Todo, 0),
		}
		// new_rem.Todos = append(new_rem.Todos, Todo{"hello", true})
		rem_json, err := json.Marshal(new_rem)
		if err != nil {
			log.Fatalln(err)
		}
		// log.Printf("rem_json: %s", rem_json)
		_, err = new_file.Write(rem_json)
		if err != nil {
			log.Fatalln(err)
		}
		// log.Println("bytes written: ", write)

		// file, err = os.Open(file_location)
		file, err = os.OpenFile(file_location, os.O_RDWR, 0644)
		if err != nil {
			log.Fatalln("Error reopening file:", err)
		}
	}
	defer file.Close()

	buf, err := io.ReadAll(file)
	if err != nil {
		log.Fatalln(err)
	}
	// log.Println("file contents: ", string(buf))

	rem := &Rem{}
	err = json.Unmarshal(buf, rem)
	if err != nil {
		log.Fatalln(err)
	}

	new_todo := flag.String("new", "", "Create a new todo")
	done_mark := flag.Int("do", -1, "Create a new todo")
	flag.Parse()

	defer printUndone(rem)
	// defer saveRemFile(rem, file)

	if *new_todo != "" && *done_mark != -1 {
		log.Fatal("Only one operation at a time")
	}

	if *new_todo != "" {
		// log.Println(new_todo)
		// rem.Todos = append(rem.Todos, Todo{Content: *new_todo, Done: false})
		rem.addTodo(*new_todo)
		saveRemFile(rem, file)
	}

	if *done_mark != -1 {
		// log.Println(new_todo)
		rem.markDone(*done_mark)
		saveRemFile(rem, file)
	}
}

func (rem *Rem) addTodo(content string) {
	rem.Todos = append(rem.Todos, Todo{Content: content, Done: false})
}

func (rem *Rem) markDone(i int) {
	if i < 1 || i > len(rem.Todos) {
		log.Fatalln("You need to enter a between 1 and ", len(rem.Todos))
	}

	rem.Todos[i-1].Done = true
	return
}

func printUndone(rem *Rem) {
	for v := range rem.Todos {
		if !rem.Todos[v].Done {
			fmt.Printf("%d. %s\n", v+1, rem.Todos[v].Content)
		}
	}
}

func saveRemFile(rem *Rem, file *os.File) {
	rem_json, err := json.Marshal(rem)
	if err != nil {
		log.Fatalln(err)
	}
	// log.Println("rem_json:\n", string(rem_json))

	err = file.Truncate(0)
	_, err = file.Seek(0, io.SeekStart)
	if err != nil {
		log.Fatalln("Error seeking to start of file:", err)
	}
	_, err = file.Write(rem_json)
	// log.Println("write: ", write)
}
