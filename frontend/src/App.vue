
<template>
  <div class="app-container" :class="{ 'sidebar-collapsed': isSidebarCollapsed }">
    <button class="sidebar-toggle-btn" @click="toggleSidebar">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path v-if="isSidebarCollapsed" d="M10 17L15 12L10 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path v-else d="M14 7L9 12L14 17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <ProjectList 
      :is-collapsed="isSidebarCollapsed"
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
const projectList = ref(null);
const isSidebarCollapsed = ref(false);

let debounceTimer = null;

const handleProjectSelected = async (projectId) => {
  try {
    const project = await getProjectById(projectId);
    let parsedContent;

    if (typeof project.content === 'string') {
      try {
        parsedContent = JSON.parse(project.content);
      } catch (e) {
        console.error("Error parsing project content:", e);
        parsedContent = { elements: [], appState: {} };
      }
    } else {
      parsedContent = project.content || { elements: [], appState: {} };
    }

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
  if (projectList.value) {
    projectList.value.fetchProjects();
  }
  handleProjectSelected(newProject.id);
};

const handleDrawingChange = ({ elements, appState, files }) => {
  if (!currentProject.value) return;

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
  }, 500);
};

const toggleSidebar = () => {
  isSidebarCollapsed.value = !isSidebarCollapsed.value;
};
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  position: relative;
  transition: padding-left 0.3s ease;
}

.sidebar-toggle-btn {
  position: absolute;
  top: 50%;
  left: calc(var(--sidebar-width, 250px) - 16px);
  transform: translateY(-50%);
  z-index: 100;
  border: 1px solid #e0e0e0;
  background: #ffffff;
  border-radius: 50%;
  width: 32px;
  height: 32px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  transition: all 0.3s ease;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  color: #666;
}

.sidebar-toggle-btn:hover {
  background: #f5f5f5;
  border-color: #ccc;
}

.app-container.sidebar-collapsed {
  --sidebar-width: 0px;
}

.app-container.sidebar-collapsed .sidebar-toggle-btn {
  left: 15px;
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
