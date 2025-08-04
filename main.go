package main

import (
	"embed"
	"encoding/json"
	"fmt"
	"io/fs"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"venus/config"
	"venus/util/jwt"

	"github.com/gin-gonic/gin"
)

//go:embed frontend/dist/*
var embeddedFiles embed.FS

const projectsFile = "projects.json"

// Project represents the data structure for a drawing project.
type Project struct {
	ID      string      `json:"id"`
	Name    string      `json:"name"`
	Content interface{} `json:"content"` // Store Excalidraw scene data as a JSON object
	Uid     uint        `json:"uid"`
}

// In-memory database
var (
	projects = make(map[string]Project)
	nextID   = 1
	mu       sync.RWMutex
)

func init() {
	loadProjects()
}

func loadProjects() {
	mu.Lock()
	defer mu.Unlock()

	data, err := os.ReadFile(projectsFile)
	if err != nil {
		if os.IsNotExist(err) {
			fmt.Println("projects.json not found, starting with empty projects.")
			projects = make(map[string]Project)
			nextID = 1
			return
		}
		fmt.Printf("Error reading projects.json: %v\n", err)
		return
	}

	var loadedProjects []Project
	if err := json.Unmarshal(data, &loadedProjects); err != nil {
		fmt.Printf("Error unmarshaling projects.json: %v\n", err)
		return
	}

	projects = make(map[string]Project)
	maxID := 0
	for _, p := range loadedProjects {
		projects[p.ID] = p
		idNum := 0
		fmt.Sscanf(p.ID, "%d", &idNum)
		if idNum > maxID {
			maxID = idNum
		}
	}
	nextID = maxID + 1
	fmt.Printf("Loaded %d projects. Next ID: %d\n", len(projects), nextID)
}

func saveProjects() {
	projectList := make([]Project, 0, len(projects))
	for _, p := range projects {
		projectList = append(projectList, p)
	}

	data, err := json.MarshalIndent(projectList, "", "  ")
	if err != nil {
		fmt.Printf("Error marshaling projects: %v\n", err)
		return
	}

	if err := os.WriteFile(projectsFile, data, 0644); err != nil {
		fmt.Printf("Error writing projects.json: %v\n", err)
	}
}

func main() {
	r := gin.Default()
	r.Use(AuthRequired())

	// Create a sub-filesystem that looks for files in the "frontend/dist" directory
	distFS, err := fs.Sub(embeddedFiles, "frontend/dist")
	if err != nil {
		panic(err)
	}

	// Create a file server handler
	fileServer := http.FileServer(http.FS(distFS))

	// Handle static files and SPA fallback
	r.NoRoute(func(c *gin.Context) {
		// Check if the request path is for an API endpoint
		if strings.HasPrefix(c.Request.URL.Path, "/api") {
			c.Next() // Continue to the next handler (which would be 404 if no API route matches)
			return
		}

		// Try to serve the file directly from the embedded filesystem
		fileServer.ServeHTTP(c.Writer, c.Request)

		// If http.FileServer didn't find the file (e.g., for SPA routes), then serve index.html
		if c.Writer.Status() == http.StatusNotFound {
			indexData, err := embeddedFiles.ReadFile("frontend/dist/index.html")
			if err != nil {
				c.String(http.StatusNotFound, "Not Found")
				return
			}
			c.Data(http.StatusOK, "text/html; charset=utf-8", indexData)
		}
	})

	api := r.Group("/api")
	{
		api.GET("/projects", getProjects)
		api.POST("/projects", createProject)
		api.GET("/projects/:id", getProjectByID)
		api.PUT("/projects/:id", updateProject)
		api.DELETE("/projects/:id", deleteProject)
	}

	r.Run(":8085")
}

// getProjects returns a list of all projects (without content for brevity).
func getProjects(c *gin.Context) {
	uid, exists := c.Get("uid")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	mu.RLock()
	defer mu.RUnlock()

	projectList := make([]Project, 0, len(projects))
	for _, p := range projects {
		if p.Uid == uid.(uint) {
			projectList = append(projectList, Project{ID: p.ID, Name: p.Name})
		}
	}

	c.JSON(http.StatusOK, projectList)
}

// createProject creates a new project.
func createProject(c *gin.Context) {
	uid, exists := c.Get("uid")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	mu.Lock()
	defer mu.Unlock()

	var req struct {
		Name string `json:"name"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	id := fmt.Sprintf("%d", nextID)
	nextID++

	newProject := Project{
		ID:   id,
		Name: req.Name,
		Content: map[string]interface{}{
			"elements": []interface{}{},
			"appState": map[string]interface{}{"collaborators": []interface{}{}},
			"files":    map[string]interface{}{},
		},
		Uid: uid.(uint),
	}
	projects[id] = newProject
	saveProjects()

	c.JSON(http.StatusCreated, newProject)
}

// getProjectByID returns the full data for a single project.
func getProjectByID(c *gin.Context) {
	uid, exists := c.Get("uid")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	mu.RLock()
	defer mu.RUnlock()

	id := c.Param("id")
	project, exists := projects[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Project not found"})
		return
	}
	if project.Uid != uid {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}
	c.JSON(http.StatusOK, project)
}

// updateProject updates a project's content.
func updateProject(c *gin.Context) {
	uid, exists := c.Get("uid")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}
	mu.Lock()
	defer mu.Unlock()

	id := c.Param("id")
	project, exists := projects[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Project not found"})
		return
	}
	if project.Uid != uid {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	var req struct {
		Content json.RawMessage `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request body"})
		return
	}

	// Unmarshal the content into an interface{}
	var contentData interface{}
	if err := json.Unmarshal(req.Content, &contentData); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid content format"})
		return
	}

	// Update the project
	p := projects[id]
	p.Content = contentData
	projects[id] = p
	saveProjects()

	c.JSON(http.StatusOK, gin.H{"status": "success"})
}

// deleteProject removes a project.
func deleteProject(c *gin.Context) {
	uid, exists := c.Get("uid")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}
	mu.Lock()
	defer mu.Unlock()

	id := c.Param("id")
	project, exists := projects[id]
	if !exists {
		c.JSON(http.StatusNotFound, gin.H{"error": "Project not found"})
		return
	}
	if project.Uid != uid {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized"})
		return
	}

	delete(projects, id)
	saveProjects()
	c.Status(http.StatusNoContent)
}

// AuthRequired 需要登录
func AuthRequired() gin.HandlerFunc {
	return func(c *gin.Context) {
		host := c.Request.Host
		if strings.HasPrefix(host, "localhost") {
			c.Set("uid", uint(1))
			c.Next()
			return
		}
		uid, err := jwt.Get(c)
		if err == nil {
			c.Set("uid", uid)
			c.Next()
			return
		}
		scheme := "http"
		if c.Request.TLS != nil {
			scheme = "https"
		}
		fullReturnURL := scheme + "://" + host
		loginURL := config.AUTH_URL + "?url=" + url.QueryEscape(fullReturnURL)
		c.Redirect(http.StatusTemporaryRedirect, loginURL)
		c.Abort()
	}
}
