<template>
    <div class="message-item">
        <div v-if="message.sender === 'bot'" class="markdown-content">
            <vue-markdown :source="message.content" :options="{
                breaks: true,
                linkify: true,
                highlight: (code: string, lang: string) => highlightCode(code, lang)
            }" />
        </div>
        <div v-else class="user-content">
            {{ message.content }}
        </div>
    </div>
</template>

<script setup lang="ts">
import VueMarkdown from 'vue-markdown-render';
import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';
import type { Message } from '../../types';

defineProps<{
    message: Message;
}>();

const highlightCode = (code: string, lang: string) => {
    if (lang && hljs.getLanguage(lang)) {
        try {
            return hljs.highlight(code, { language: lang }).value;
        } catch (e) {
            console.error(e);
        }
    }
    return hljs.highlightAuto(code).value;
};
</script>

<style scoped>
.message-item {
    font-size: 14px;
    line-height: 1.5;
}

.markdown-content {
    color: #333;
}

.markdown-content :deep(pre) {
    background-color: #282c34;
    border-radius: 6px;
    padding: 12px;
    overflow-x: auto;
}

.markdown-content :deep(code) {
    font-family: 'Fira Code', monospace;
    font-size: 13px;
}

.markdown-content :deep(p) {
    margin: 0.5em 0;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
    padding-left: 1.5em;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4) {
    margin: 0.5em 0;
}

.markdown-content :deep(table) {
    border-collapse: collapse;
    width: 100%;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
    border: 1px solid #ddd;
    padding: 8px;
    text-align: left;
}

.markdown-content :deep(th) {
    background-color: #f2f2f2;
}

.user-content {
    color: #fff;
}

.user-content a {
    color: #fff;
    text-decoration: underline;
}
</style>