
<template>
  <div class="project-sidebar">
    <h2>Projects</h2>
    <div class="project-list">
      <div 
        v-for="project in projects" 
        :key="project.id"
        class="project-item"
        :class="{ active: project.id === selectedProjectId }"
        @click="selectProject(project.id)">
        <span>{{ project.name }}</span>
        <button class="delete-btn" @click.stop="deleteProject(project.id)">X</button>
      </div>
    </div>
    <div class="new-project-form">
      <input v-model="newProjectName" placeholder="New Project Name" @keyup.enter="create"/>
      <button @click="create">Create</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, defineEmits, defineExpose } from 'vue';
import { getProjects, createProject, deleteProject as apiDeleteProject } from '../api/projects';

const projects = ref([]);
const newProjectName = ref('');
const selectedProjectId = ref(null);

const emit = defineEmits(['project-selected', 'new-project']);

const fetchProjects = async () => {
  try {
    projects.value = await getProjects();
  } catch (error) {
    console.error("Failed to fetch projects:", error);
  }
};

const selectProject = (id) => {
  selectedProjectId.value = id;
  emit('project-selected', id);
};

const create = async () => {
  if (!newProjectName.value.trim()) return;
  try {
    const newProject = await createProject({ name: newProjectName.value });
    newProjectName.value = '';
    await fetchProjects(); // Refresh the list
    emit('new-project', newProject);
    selectProject(newProject.id); // Auto-select the new project
  } catch (error) {
    console.error("Failed to create project:", error);
  }
};

const deleteProject = async (id) => {
  if (!confirm("Are you sure you want to delete this project?")) return;
  try {
    await apiDeleteProject(id);
    await fetchProjects(); // Refresh the list
    if (selectedProjectId.value === id) {
      selectedProjectId.value = null;
      emit('project-selected', null); // Clear selection
    }
  } catch (error) {
    console.error("Failed to delete project:", error);
  }
};

onMounted(fetchProjects);

// Expose fetchProjects to be called from parent
defineExpose({ fetchProjects });

</script>

<style scoped>
.project-sidebar {
  width: 250px;
  border-right: 1px solid #ccc;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  background-color: #f9f9f9;
}

h2 {
  margin-top: 0;
  border-bottom: 1px solid #eee;
  padding-bottom: 0.5rem;
}

.project-list {
  flex-grow: 1;
  overflow-y: auto;
}

.project-item {
  padding: 0.75rem;
  cursor: pointer;
  border-radius: 4px;
  margin-bottom: 0.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.project-item:hover {
  background-color: #e0e0e0;
}

.project-item.active {
  background-color: #cce5ff;
  font-weight: bold;
}

.delete-btn {
  background: none;
  border: none;
  color: #ff4d4d;
  cursor: pointer;
  font-weight: bold;
  visibility: hidden; /* Hide by default */
}

.project-item:hover .delete-btn {
  visibility: visible; /* Show on hover */
}

.new-project-form {
  margin-top: 1rem;
  display: flex;
}

.new-project-form input {
  flex-grow: 1;
  padding: 0.5rem;
  border: 1px solid #ccc;
  border-radius: 4px 0 0 4px;
}

.new-project-form button {
  padding: 0.5rem 1rem;
  border: 1px solid #007bff;
  background-color: #007bff;
  color: white;
  border-radius: 0 4px 4px 0;
  cursor: pointer;
}

.new-project-form button:hover {
  background-color: #0056b3;
}
</style>
