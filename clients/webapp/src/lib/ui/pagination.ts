interface PaginationResult {
    start: number;
    end: number;
    prev: number;
    next: number;
    total: number;
    schema: number[];
}

function generatePaginationSchema(total: number, currentPage: number, pageSize: number): number[] {
    const totalPages = Math.ceil(total / pageSize);
    const arr: number[] = [];

    if (totalPages <= 7) {
        for (let i = 1; i <= totalPages; i++) arr.push(i);
    } else {
        arr.push(1);
        if (currentPage < 4) {
            for (let i = 2; i <= 4; i++) arr.push(i);
            arr.push(0);
        }
        if (currentPage >= 4 && currentPage <= totalPages - 3) {
            arr.push(0);
            for (let i = currentPage - 1; i <= currentPage + 1; i++) arr.push(i);
            arr.push(0);
        }
        if (currentPage > totalPages - 3) {
            arr.push(0);
            for (let i = totalPages - 3; i <= totalPages - 1; i++) arr.push(i);
        }
        arr.push(totalPages);
    }

    return arr;
}

export function pagination(total: number, currentPage: number, pageSize: number): PaginationResult {
    const start = (currentPage - 1) * pageSize + 1;
    const end = currentPage * pageSize > total ? total : currentPage * pageSize;
    const prev = currentPage > 1 ? currentPage - 1 : 1;
    const next =
        currentPage < Math.ceil(total / pageSize)
            ? currentPage + 1
            : Math.ceil(total / pageSize);
    const schema = generatePaginationSchema(total, currentPage, pageSize);
    return { start, end, prev, next, total, schema };
}
