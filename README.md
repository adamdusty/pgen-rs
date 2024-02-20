# pgen

Simple project generator developed for my personal use. Starting new c/c++ projects with cmake is horrible due to all the cmake boilerplate. This utility can generate a template from a directory, and generate a directory/files from a template. There is no interactive mode yet so template variables have to be supplied in and read from a file. I re-wrote this utility in rust originally as a learning experience, but I like it better than the c++ version I made.

## Usage

`pgen gen destination --template path/to/template.yaml --definitions path/to/template_defs.yaml`  
`pgen fd directory --output path/to/template.yaml --force (overwrite output path if it exists)`
