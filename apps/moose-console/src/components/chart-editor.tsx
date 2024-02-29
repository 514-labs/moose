"use client"
import { useEffect } from 'react'
import { TopLevelSpec } from 'vega-lite'
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from './ui/select'
import { useForm } from 'react-hook-form'
import { Form, FormControl, FormField, FormItem, FormLabel } from './ui/form'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { Dispatch, SetStateAction } from 'react'

const test_form = {
    x: { field: 'timestamp', type: 'temporal' },
    y: { field: 'eventName', type: 'nominal' },
    color: { field: 'eventType', type: 'nominal' },
    tooltip: { field: 'sessionId', type: 'nominal' },
}

const fields = ["eventName", "timestamp", "eventType"]


const EncodingForm = ({ formPath, options, name }: { name: string, formPath: string, options: string[] }) => {
    return (
        <FormField name={formPath} render={({ field }) => {
            return <FormItem className='flex items-center'>
                <FormLabel >{name}</FormLabel>
                <FormControl>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <SelectTrigger className='w-[180px]'>
                            <SelectValue placeholder={"Field"} />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectGroup>
                                {options.map((option, i) => (
                                    <SelectItem key={i} value={option}>{option}</SelectItem>
                                ))}
                            </SelectGroup>
                        </SelectContent>
                    </Select>
                </FormControl></FormItem>
        }} />
    )
}

export const MarkForm = ({ formPath, options, name }: { name: string, formPath: string, options: string[] }) => {
    return (
        <FormField name={formPath} render={({ field }) => {
            return <FormItem className='flex items-center w-20'>
                <FormLabel >{name}</FormLabel>
                <FormControl>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <SelectTrigger className='w-[180px]'>
                            <SelectValue placeholder={"Field"} />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectGroup>
                                {options.map((option, i) => (
                                    <SelectItem key={i} value={option}>{option}</SelectItem>
                                ))}
                            </SelectGroup>
                        </SelectContent>
                    </Select>
                </FormControl></FormItem>
        }} />
    )
}

const InputForm = ({ formPath, name }: { name: string, formPath: string }) => {
    return (
        <FormField name={formPath} render={({ field }) => {
            return <FormItem className='flex items-center'>
                <FormLabel >{name}</FormLabel>
                <FormControl>
                    <Input {...field} />
                </FormControl></FormItem>
        }} />
    )
}

function SpecForm({ spec, setSpec, keys }: { spec: TopLevelSpec, keys: string[], setSpec: (spec: TopLevelSpec) => void }) {

    const fake_keys = Object.keys(test_form)



    const { data, ...otherSpec } = spec
    console.log(otherSpec)
    return (

                <div className='grid grid-cols-2'>
                    {fake_keys.map((key, i) => <EncodingForm key={i} formPath={`encoding.${key}.field`} options={fields} name={key} />)}
                </div>
    
    )
}


export function ChartEditor({ spec, setSpec, keys }: { spec: TopLevelSpec, keys: string[], setSpec: Dispatch<SetStateAction<TopLevelSpec>> }) {
    return <SpecForm setSpec={setSpec} spec={spec} keys={keys} />
}