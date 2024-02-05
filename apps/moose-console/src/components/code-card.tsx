"use client"

import { codeToHtml } from 'shiki'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "components/ui/select"
import { Card, CardContent } from "./ui/card";
import { useEffect, useState } from "react";
import parse from 'html-react-parser'
import { Button } from './ui/button';


interface Snippet {
    language: string;
    code: string;
}

interface CodeCardProps {
    title: string;
    snippets: Snippet[];
}

export default function CodeCard({ title, snippets }: CodeCardProps) {
    const [selectedSnippet, setSelectedSnippet] = useState<Snippet>(snippets[0]);
    const [formatedCodeSnippets, setFormatedCodeSnippets] = useState<{language?: string, formatedSnippet?: string}>({})

    useEffect(() => {
        const formatedCode = async () => {
            const newFormatedCodeSnippets = {}

            for (const snippet of snippets) {
                const html = await codeToHtml(snippet.code, { lang: snippet.language, theme: 'none' })
                newFormatedCodeSnippets[snippet.language] = html
            }

            setFormatedCodeSnippets(newFormatedCodeSnippets)
        }
        formatedCode();
    }, [snippets])

    return (
        <div>
            <div className="flex flex-row items-center py-2">
                <h2>
                    {title}
                </h2>
                <span className="grow"/>
                <Button variant="outline" className="mr-2" onClick={() => {
                    navigator.clipboard.writeText(selectedSnippet.code)
                }}>copy</Button>
                <div>
                
                <Select defaultValue={selectedSnippet.language} onValueChange={
                    (value) => {
                        const snippet = snippets.find(snippet => snippet.language === value)
                        console.log(formatedCodeSnippets)
                        if (snippet) {
                            setSelectedSnippet(snippet)
                        }
                    }
                }>
                    <SelectTrigger>
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {snippets.map((snippet, index) => (
                            <SelectItem
                                key={index}
                                value={snippet.language}
                                onClick={() => setSelectedSnippet(snippet)}
                            >
                                {snippet.language}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                </div>
            </div>
            <Card className="rounded-2xl bg-muted ">
                <CardContent className=''>
                    <code>
                        {formatedCodeSnippets[selectedSnippet.language] ? parse(formatedCodeSnippets[selectedSnippet.language]) : "Loading..."}
                    </code>
                </CardContent>
            </Card>
        </div>
    )
}