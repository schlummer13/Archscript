# specs/dsl-v0.1.md

# ArchScript DSL v0.1

## File extension

`.archs`

## Basic syntax

```archscript
project SolagoPortal

element actor Customer
element frontend WebApp
element backend API
element database PostgreSQL

relation Customer -> WebApp : uses
relation WebApp -> API : REST
relation API -> PostgreSQL : reads/writes