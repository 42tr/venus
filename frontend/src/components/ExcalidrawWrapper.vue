<template>
  <div ref="excalidrawContainer" style="height: 100%; width: 100%;"></div>
</template>

<script setup>
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { createRoot } from 'react-dom/client';
import { createElement } from 'react';
import { Excalidraw } from '@excalidraw/excalidraw';
import { uploadImage } from '../api/images';
import getApiConfig from '../config/api.js';

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
let fileIdToData = new Map();
let isInitializing = false;

const apiConfig = getApiConfig();

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

// 处理文件上传和清理
const handleFileProcessing = async (files) => {
  const processedFiles = {};

  for (const [fileId, fileData] of Object.entries(files)) {
    if (uploadedFileIds.has(fileId)) {
      // 已上传文件，使用缓存的清理后数据
      const cleanData = fileIdToData.get(fileId);
      if (cleanData) {
        processedFiles[fileId] = {
          ...fileData,
          ...cleanData,
        };
      } else {
        processedFiles[fileId] = fileData;
      }
      continue;
    }

    if (fileData.dataURL && fileData.dataURL.startsWith('data:image/')) {
      // 新的 Base64 图片，执行上传
      try {
        const blob = base64ToBlob(fileData.dataURL, fileData.mimeType);
        const file = new File([blob], `pasted-image-${Date.now()}.png`, {
          type: fileData.mimeType || 'image/png',
        });

        const uploadResult = await uploadImage(file, props.projectId);

        const newData = {
          mimeType: fileData.mimeType,
          created: fileData.created || Date.now(),
          lastRetrieved: Date.now(),
          imageId: uploadResult.id,
          dataURL: `/api/images/${uploadResult.id}`,
          uploaded: true,
        };

        processedFiles[fileId] = newData;
        uploadedFileIds.add(fileId);
        fileIdToData.set(fileId, newData);
      } catch (error) {
        // 上传失败则不包含此文件
      }
    } else {
      // 非 Base64 文件，直接保留
      processedFiles[fileId] = fileData;
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
    processedFiles = await handleFileProcessing(files);
  }

  emit('change', { elements, appState, files: processedFiles });
};

onMounted(() => {
  isInitializing = true;

  uploadedFileIds.clear();
  fileIdToData.clear();

  const processedInitialData = JSON.parse(JSON.stringify(props.initialData || {}));

  if (processedInitialData.files) {
    const fileIdMapping = {};
    Object.entries(processedInitialData.files).forEach(([fileId, fileData]) => {
      const isReallyUploaded =
        fileData.uploaded &&
        fileData.imageId &&
        fileData.dataURL &&
        fileData.dataURL.startsWith('/api/images/');

      if (isReallyUploaded) {
        uploadedFileIds.add(fileId);
        fileIdToData.set(fileId, {
          imageId: fileData.imageId,
          dataURL: fileData.dataURL,
          mimeType: fileData.mimeType,
          created: fileData.created,
          uploaded: true,
        });
        
        fileData.dataURL = `${apiConfig.imageBaseURL}${fileData.dataURL}`;

        if (fileData.imageId && fileData.imageId !== fileId) {
          fileIdMapping[fileData.imageId] = fileId;
        }
      }
    });

    if (processedInitialData.elements) {
      processedInitialData.elements.forEach((element) => {
        if (element.type === 'image' && element.fileId) {
          if (fileIdMapping[element.fileId]) {
            element.fileId = fileIdMapping[element.fileId];
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
  fileIdToData.clear();
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