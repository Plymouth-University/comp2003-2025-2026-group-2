<button type="button" class="log_button" aria-label="Previous Log">
	<span class="mb-6 text-left text-2xl font-bold">Previous Log</span>
</button>

<style>
	.log_button {
		position: absolute;
		transform: translate(-50%, 0%);
		left: 50%;
		display: inline-block;
		border-radius: 4px;
		border-width: 2px;
		border-color: #000100;
		background-color: #ffffff;
		color: #000100;
		text-align: center;
		font-size: 28px;
		padding: 20px;
		padding-top: 30px;
		width: 220px;
		cursor: pointer;
		transition: 0.5s;
		vertical-align: middle;
	}

	.log_button span {
		display: inline-block;
		position: relative;
		transition: 0.5s;
	}

	.log_button span:after {
		content: '\00ab';
		position: absolute;
		opacity: 0;
		left: -10px;
		transition: 0.5s;
	}

	.log_button:hover span {
		padding-left: 25px;
	}

	.log_button:hover span:after {
		opacity: 1;
		right: 0;
	}

	.log_button:hover {
		background-color: #ededf2;
	}
</style>
