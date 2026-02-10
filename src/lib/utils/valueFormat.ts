export function smartFormat(value: any): { text: string; isComplex: boolean } {
	if (value === null || value === undefined) return { text: '', isComplex: false };

	if (Array.isArray(value)) {
		if (value.length === 0) return { text: '[]', isComplex: true };

		// heuristic: check first item for common display keys
		const first = value[0];
		if (typeof first === 'object' && first !== null) {
			const displayKeys = [
				'name',
				'title',
				'label',
				'id',
				'slug',
				'email',
				'username',
				'code',
				'key',
				'status'
			];
			const hit = displayKeys.find((k) => k in first);
			if (hit) {
				return {
					text: value.map((v: any) => (v && v[hit])).join(', '),
					isComplex: true
				};
			}
		}

		// fallback for mixed arrays or objects without common keys
		const text = value
			.map((v: any) => {
				if (typeof v === 'object' && v !== null) return JSON.stringify(v);
				return String(v);
			})
			.join(', ');
		return { text, isComplex: true };
	}

	if (typeof value === 'object') {
		const displayKeys = [
			'name',
			'title',
			'label',
			'id',
			'slug',
			'email',
			'username',
			'code',
			'key',
			'status'
		];
		const hit = displayKeys.find((k) => k in value);
		if (hit) {
			return { text: String(value[hit]), isComplex: true };
		}
		return { text: JSON.stringify(value), isComplex: true };
	}

	return { text: String(value), isComplex: false };
}

// Helper to safely get value for a nested column path (e.g. "user_name")
export function getValue(parsed: any, colPath: string): { text: string; isComplex: boolean } {
	if (!parsed || typeof parsed !== 'object') return { text: '', isComplex: false };

	const parts = colPath.split('_');
	let current = parsed;

	for (const part of parts) {
		if (current === null || current === undefined || typeof current !== 'object') {
			return { text: '', isComplex: false };
		}
		current = current[part];
	}

	if (current === undefined || current === null) return { text: '', isComplex: false };
	return smartFormat(current);
}
