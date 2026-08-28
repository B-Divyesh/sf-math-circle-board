import {defineConfig} from 'vite';
export default defineConfig({
  root: '.',
  publicDir: 'frontend/public',
  build: {outDir:'dist',emptyOutDir:true,target:'es2022'},
  server: {proxy:{'/api':'http://localhost:8080','/health':'http://localhost:8080'}}
});
