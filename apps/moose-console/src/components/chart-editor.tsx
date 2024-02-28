"use client"

import { TopLevelSpec } from 'vega-lite'
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from './ui/select'
import { useForm } from 'react-hook-form'
import { Form, FormControl, FormField, FormItem, FormLabel } from './ui/form'
import { Button } from './ui/button'
import { Input } from './ui/input'

const test_form = {
    x: { field: 'Horsepower', type: 'quantitative' },
    y: { field: 'Miles_per_Gallon', type: 'quantitative' },
    color: { field: 'Origin', type: 'nominal' },
    tooltip: { field: 'Name', type: 'nominal' },
}

const fields = ["Horsepower", "Miles_per_Gallon", "Origin", "Name", "url"]


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

const MarkForm = ({ formPath, options, name }: { name: string, formPath: string, options: string[] }) => {
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

function SpecForm({ spec, setSpec }: { spec: TopLevelSpec, setSpec: (spec: TopLevelSpec) => void }) {
    const methods = useForm({ defaultValues: spec })
    const { handleSubmit, register } = methods;
    const onSubmit = (data) => setSpec(data)
    const keys = Object.keys(test_form)

    return (
        <Form {...methods}>
            <form onSubmit={handleSubmit(onSubmit)}>
                <div className='grid grid-cols-2'>
                <InputForm formPath='height' name={"height"} />
                <InputForm formPath='width' name={"width"} />

                <MarkForm options={['line', 'bar', 'circle']} formPath={'mark'} name={"Type"} />
                {keys.map((key, i) => <EncodingForm key={i} formPath={`encoding.${key}.field`} options={fields} name={key} />)}
                </div>
                <Button type="submit" >Submit</Button>
            </form>
        </Form>
    )
}


export function ChartEditor({ spec, setSpec }: { spec: TopLevelSpec, setSpec: (spec: TopLevelSpec) => void }) {
    return <SpecForm setSpec={setSpec} spec={spec} />
}