
<template>
  <div class="app-container">
    <ProjectList 
      @project-selected="handleProjectSelected" 
      @new-project="handleNewProject"
      ref="projectList"
    />
    <div class="excalidraw-wrapper">
      <ExcalidrawWrapper 
        v-if="currentProject" 
        :key="currentProject.id" 
        :initial-data="currentProject.content"
        @change="handleDrawingChange"
      />
      <div v-else class="no-project-selected">
        <h2>Select a project or create a new one</h2>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import ProjectList from './components/ProjectList.vue';
import ExcalidrawWrapper from './components/ExcalidrawWrapper.vue';
import { getProjectById, updateProject } from './api/projects';

const currentProject = ref(null);
const projectList = ref(null); // Ref to access ProjectList component methods

let debounceTimer = null;

const handleProjectSelected = async (projectId) => {
  try {
    const project = await getProjectById(projectId);
    let parsedContent;

    // Parse content if it's a string, otherwise use as is
    if (typeof project.content === 'string') {
      try {
        parsedContent = JSON.parse(project.content);
      } catch (e) {
        console.error("Error parsing project content:", e);
        parsedContent = { elements: [], appState: {} }; // Fallback on error
      }
    } else {
      parsedContent = project.content || { elements: [], appState: {} };
    }

    // Ensure appState and collaborators array exist to prevent Excalidraw error
    if (!parsedContent.appState) {
      parsedContent.appState = {};
    }
    if (!Array.isArray(parsedContent.appState.collaborators)) {
      parsedContent.appState.collaborators = [];
    }

    project.content = parsedContent;
    currentProject.value = project;
  } catch (error) {
    console.error("Failed to load project:", error);
  }
};

const handleNewProject = (newProject) => {
  // This will refresh the list and optionally select the new project
  if (projectList.value) {
    projectList.value.fetchProjects();
  }
  // Automatically select the new project to start drawing
  handleProjectSelected(newProject.id);
};

const handleDrawingChange = ({ elements, appState, files }) => {
  if (!currentProject.value) return;

  // Debounce the save operation
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(async () => {
    try {
      const contentToSave = { elements, appState, files };
      await updateProject(currentProject.value.id, { content: JSON.stringify(contentToSave) });
      console.log("Project saved!");
    } catch (error) {
      console.error("Failed to save project:", error);
    }
  }, 500); // Save after 500ms of inactivity
};

</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
}

.excalidraw-wrapper {
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #f0f0f0;
}

.no-project-selected {
  text-align: center;
  color: #888;
}
</style>
