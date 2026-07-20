/* tslint:disable */
/* eslint-disable */

export enum Cell {
    Dead = 0,
    Alive = 1,
}

export class Universe {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cells(): number;
    height(): number;
    static new(): Universe;
    tick(): void;
    width(): number;
}
