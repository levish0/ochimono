/* tslint:disable */
/* eslint-disable */

export class Game {
    free(): void;
    [Symbol.dispose](): void;
    advance(ticks: number): void;
    configure(gravity: boolean, handling: string, entry_delay: number): void;
    input(input: string): void;
    constructor(seed: string, mode: string, gravity: boolean, handling: string, entry_delay: number);
    redo(): boolean;
    release_inputs(): void;
    restore(snapshot: string): void;
    snapshot(): string;
    undo(): boolean;
    view(): string;
}
