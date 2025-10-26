<template>
  <div ref="excalidrawContainer" style="height: 100%; width: 100%;"></div>
</template>

<script setup>
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { createRoot } from 'react-dom/client';
import { createElement } from 'react';
import { Excalidraw } from '@excalidraw/excalidraw';
import { uploadImage } from '../api/images';

const props = defineProps({
  initialData: {
    type: Object,
    default: () => ({ elements: [], appState: {} }),
  },
  projectId: {
    type: String,
    default: null,
  },
});

const emit = defineEmits(['change']);

const excalidrawContainer = ref(null);
let root = null;
let uploadedFileIds = new Set();
let isInitializing = false;

// 将 base64 转换为 Blob
const base64ToBlob = (base64, mimeType) => {
  const byteCharacters = atob(base64.split(',')[1]);
  const byteNumbers = new Array(byteCharacters.length);
  for (let i = 0; i < byteCharacters.length; i++) {
    byteNumbers[i] = byteCharacters.charCodeAt(i);
  }
  const byteArray = new Uint8Array(byteNumbers);
  return new Blob([byteArray], { type: mimeType });
};

// 处理文件上传
const handleFileUpload = async (files) => {
  const processedFiles = {};
  
  for (const [fileId, fileData] of Object.entries(files)) {
    if (uploadedFileIds.has(fileId)) {
      processedFiles[fileId] = fileData;
      continue;
    }
    
    if (fileData.dataURL && fileData.dataURL.startsWith('data:image/')) {
      try {
        const blob = base64ToBlob(fileData.dataURL, fileData.mimeType);
        const file = new File([blob], `pasted-image-${Date.now()}.png`, {
          type: fileData.mimeType || 'image/png'
        });

        const uploadResult = await uploadImage(file, props.projectId);
        const baseURL = 'http://localhost:8085';
        const fullImageURL = uploadResult.url.startsWith('http') 
          ? uploadResult.url 
          : `${baseURL}${uploadResult.url}`;
        
        processedFiles[fileId] = {
          ...fileData,
          dataURL: fullImageURL,
          id: uploadResult.id,
        };
        
        uploadedFileIds.add(fileId);
      } catch (error) {
        console.error('图片上传失败:', error);
        processedFiles[fileId] = fileData;
      }
    } else {
      processedFiles[fileId] = fileData;
      if (fileData.dataURL && !fileData.dataURL.startsWith('data:')) {
        uploadedFileIds.add(fileId);
      }
    }
  }
  
  return processedFiles;
};

const handleChange = async (elements, appState, files) => {
  if (isInitializing) {
    return;
  }
  
  let processedFiles = files;
  
  if (files && Object.keys(files).length > 0) {
    const hasNewBase64Images = Object.entries(files).some(([fileId, file]) => 
      file.dataURL && 
      file.dataURL.startsWith('data:image/') && 
      !uploadedFileIds.has(fileId)
    );
    
    if (hasNewBase64Images) {
      processedFiles = await handleFileUpload(files);
    }
  }
  
  emit('change', { elements, appState, files: processedFiles });
};

onMounted(() => {
  isInitializing = true;
  
  const processedInitialData = JSON.parse(JSON.stringify(props.initialData || {}));
  
  if (processedInitialData.files) {
    const baseURL = 'http://localhost:8085';
    const fileIdMapping = {};
    
    Object.entries(processedInitialData.files).forEach(([fileId, fileData]) => {
      if (fileData.dataURL && !fileData.dataURL.startsWith('data:')) {
        uploadedFileIds.add(fileId);
        
        if (fileData.dataURL.startsWith('/api/images/')) {
          fileData.dataURL = `${baseURL}${fileData.dataURL}`;
        }
        
        if (fileData.id && fileData.id !== fileId) {
          fileIdMapping[fileData.id] = fileId;
        }
      }
    });
    
    if (processedInitialData.elements) {
      const availableFileIds = Object.keys(processedInitialData.files || {});
      
      processedInitialData.elements.forEach((element) => {
        if (element.type === 'image' && element.fileId) {
          const oldFileId = element.fileId;
          
          if (fileIdMapping[oldFileId]) {
            element.fileId = fileIdMapping[oldFileId];
          } else if (!processedInitialData.files[oldFileId] && availableFileIds.length > 0) {
            element.fileId = availableFileIds[0];
          }
        }
      });
    }
  }

  if (excalidrawContainer.value) {
    root = createRoot(excalidrawContainer.value);
    
    const excalidrawElement = createElement(Excalidraw, {
      initialData: processedInitialData,
      onChange: handleChange,
      UIOptions: {
        canvasActions: {
          loadScene: true,
          saveScene: true,
          saveAsImage: true,
        },
      },
    });

    root.render(excalidrawElement);
    
    setTimeout(() => {
      isInitializing = false;
    }, 1500);
  }
});

watch(() => props.projectId, () => {
  uploadedFileIds.clear();
  isInitializing = true;
  
  setTimeout(() => {
    isInitializing = false;
  }, 1500);
});

onUnmounted(() => {
  if (root) {
    root.unmount();
  }
});
</script>