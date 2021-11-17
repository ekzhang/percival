package api

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"

	"github.com/google/go-github/v40/github"
	"golang.org/x/oauth2"
)

const USERNAME = "percival-bot"
const TOKEN_VAR = "GITHUB_TOKEN"

func tokenSource() oauth2.TokenSource {
	token := os.Getenv(TOKEN_VAR)
	if token == "" {
		log.Fatalf("Could not find environment variable %v for authentication", TOKEN_VAR)
	}
	return oauth2.StaticTokenSource(&oauth2.Token{AccessToken: token})
}

func Handler(w http.ResponseWriter, r *http.Request) {
	if r.Method == "GET" {
		id := r.URL.Query().Get("id")
		if id == "" {
			w.WriteHeader(http.StatusBadRequest)
			fmt.Fprint(w, "Missing `id` query field")
			return
		}

		url := fmt.Sprintf("https://gist.githubusercontent.com/%v/%v/raw", USERNAME, id)
		resp, err := http.Get(url)
		if err != nil || resp.StatusCode != http.StatusOK {
			w.WriteHeader(http.StatusNotFound)
			fmt.Fprintf(w, "Failed to fetch gist with ID %v", id)
			return
		}

		result, err := io.ReadAll(resp.Body)
		if err != nil {
			log.Fatalf("Failed to read body of gist GET response")
		}
		fmt.Fprint(w, string(result))

	} else if r.Method == "POST" {
		body, err := io.ReadAll(r.Body)
		if err != nil || len(body) == 0 {
			w.WriteHeader(http.StatusBadRequest)
			fmt.Fprint(w, "Missing body of POST request")
			return
		}

		ctx := context.Background()
		tc := oauth2.NewClient(ctx, tokenSource())

		client := github.NewClient(tc)

		public := false
		description := "Code shared from a Percival notebook - https://percival.ink"
		content := string(body)
		gist, _, err := client.Gists.Create(ctx, &github.Gist{
			Public:      &public,
			Description: &description,
			Files: map[github.GistFilename]github.GistFile{
				"notebook.percival": {Content: &content},
			},
		})
		if err != nil {
			log.Fatalf("Failed to create a gist: %v", err)
		}

		output, err := json.Marshal(gist)
		if err != nil {
			log.Fatalf("Failed to marshal result to json: %v", err)
		}

		fmt.Fprint(w, string(output))
	}
}
